use mongodb::{options::ClientOptions, Client};
use std::env;

// Global MongoDB client instance using OnceCell for lazy initialization
pub(crate) static MONGODB_CLIENT: tokio::sync::OnceCell<mongodb::Client> =
    tokio::sync::OnceCell::const_new();

/// Get the MongoDB client instance
pub async fn get_mongodb_client() -> Result<&'static Client, mongodb::error::Error> {
    MONGODB_CLIENT
        .get_or_try_init(|| async {
            let mongodb_uri =
                env::var("MONGODB_URI").unwrap_or_else(|_| "mongodb://localhost:27017".to_string());

            let client_options = ClientOptions::parse(&mongodb_uri).await?;
            let client = Client::with_options(client_options)?;

            // Test the connection
            client
                .database("admin")
                .run_command(mongodb::bson::doc! { "ping": 1 }, None)
                .await?;

            println!("Successfully connected to MongoDB at: {}", mongodb_uri);

            Ok(client)
        })
        .await
}

/// Get a database instance
pub async fn get_database(database_name: &str) -> Result<mongodb::Database, mongodb::error::Error> {
    let client = get_mongodb_client().await?;
    Ok(client.database(database_name))
}
