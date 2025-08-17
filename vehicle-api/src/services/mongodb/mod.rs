use bson::{oid::ObjectId, Document};
use futures::TryStreamExt;
use mongodb::options::CountOptions;
use mongodb::options::DeleteOptions;
use mongodb::options::FindOneAndReplaceOptions;
use mongodb::options::FindOneOptions;
use mongodb::options::FindOptions;
use mongodb::options::InsertOneOptions;
use mongodb::options::UpdateModifications;
use mongodb::options::UpdateOptions;
use mongodb::results::UpdateResult;
use mongodb::Collection;
use mongodb::{options::ClientOptions, Client};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::env;
use std::marker::Unpin;

use crate::error::{AppError, AppResult};

pub mod query_builder;
pub use query_builder::QueryBuilder;

pub const DATABASE_NAME: &str = "vehicle_booking";

// Global MongoDB client instance using OnceCell for lazy initialization
pub(crate) static MONGODB_CLIENT: tokio::sync::OnceCell<mongodb::Client> =
    tokio::sync::OnceCell::const_new();

/// Get the MongoDB client instance
pub async fn get_mongodb_client() -> AppResult<&'static Client> {
    MONGODB_CLIENT
        .get_or_try_init(|| async {
            let mongodb_uri =
                env::var("MONGODB_URI").unwrap_or_else(|_| "mongodb://localhost:27017".to_string());

            let client_options = ClientOptions::parse(&mongodb_uri).await?;
            let client = Client::with_options(client_options)?;

            // Test the connection
            client
                .database("admin")
                .run_command(mongodb::bson::doc! { "ping": 1 })
                .await?;

            println!("Successfully connected to MongoDB at: {}", mongodb_uri);

            Ok(client)
        })
        .await
}

/// Get a database instance
pub async fn get_database(database_name: &str) -> AppResult<mongodb::Database> {
    let client = get_mongodb_client().await?;
    Ok(client.database(database_name))
}

pub(crate) trait MongoStruct {
    /// Returns the name of the collection associated with the object.
    fn get_collection() -> &'static str;
}

pub(crate) async fn get_collection<T: MongoStruct + Sync + Send>(
    client: &mongodb::Client,
) -> mongodb::Collection<T> {
    client
        .database(DATABASE_NAME)
        .collection(T::get_collection())
}

pub(crate) async fn get_one<T: MongoStruct + Sync + Send + Unpin + DeserializeOwned>(
    filter: Document,
    options: impl Into<Option<FindOneOptions>>,
) -> AppResult<Option<T>> {
    let client = get_mongodb_client().await?;
    let coll = get_collection(client).await;
    coll.find_one(filter)
        .with_options(options)
        .await
        .map_err(AppError::from)
}

pub(crate) async fn get_many<T: MongoStruct + Sync + Send + Unpin + DeserializeOwned>(
    filter: Document,
    options: impl Into<Option<FindOptions>>,
) -> AppResult<mongodb::Cursor<T>> {
    let client = get_mongodb_client().await?;
    let coll = get_collection(client).await;
    coll.find(filter)
        .with_options(options)
        .await
        .map_err(AppError::from)
}

pub(crate) async fn collect_many<T: MongoStruct + Sync + Send + Unpin + DeserializeOwned>(
    filter: Document,
    options: impl Into<Option<FindOptions>>,
) -> AppResult<Vec<T>> {
    get_many(filter, options)
        .await?
        .try_collect()
        .await
        .map_err(AppError::from)
}

pub(crate) async fn insert_one<T: MongoStruct + Sync + Send + Unpin + Serialize>(
    obj: &T,
    options: impl Into<Option<InsertOneOptions>>,
) -> AppResult<ObjectId> {
    let client = get_mongodb_client().await?;
    let coll: Collection<T> = get_collection(client).await;
    let result = coll.insert_one(obj).with_options(options).await?;
    Ok(result.inserted_id.as_object_id().ok_or_else(|| {
        AppError::internal_server_error(
            "Err convert document to object id in service::insert".to_string(),
        )
    })?)
}

pub(crate) async fn delete_one(
    collection_name: &str,
    filter: Document,
    options: impl Into<Option<DeleteOptions>>,
) -> AppResult<()> {
    let client = get_mongodb_client().await?;
    let coll = client
        .database(DATABASE_NAME)
        .collection::<Document>(collection_name);
    coll.delete_one(filter).with_options(options).await?;

    Ok(())
}

/// Update.
pub(crate) async fn update_one(
    collection_name: &str,
    query: Document,
    update: impl Into<UpdateModifications>,
    options: impl Into<Option<UpdateOptions>>,
) -> AppResult<UpdateResult> {
    let client = get_mongodb_client().await?;
    let coll = client
        .database(DATABASE_NAME)
        .collection::<Document>(collection_name);
    let doc = update.into();
    coll.update_one(query, doc)
        .with_options(options)
        .await
        .map_err(AppError::from)
}

pub(crate) async fn count(
    collection_name: &str,
    filter: bson::document::Document,
    options: Option<CountOptions>,
) -> AppResult<u64> {
    let client = get_mongodb_client().await?;
    let coll = client
        .database(DATABASE_NAME)
        .collection::<Document>(collection_name);
    coll.count_documents(filter)
        .with_options(options)
        .await
        .map_err(AppError::from)
}

/// Find and replace.
pub(crate) async fn find_one_and_replace<
    T: MongoStruct + Sync + Send + Serialize + DeserializeOwned,
>(
    filter: Document,
    obj: T,
    options: impl Into<Option<FindOneAndReplaceOptions>>,
) -> AppResult<Option<T>> {
    let client = get_mongodb_client().await?;
    let coll = get_collection(client).await;
    coll.find_one_and_replace(filter, obj)
        .with_options(options)
        .await
        .map_err(AppError::from)
}
