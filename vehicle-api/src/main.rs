mod authentication;
mod controllers;
mod error;
mod models;
mod routes;
mod services;
mod util;
mod validator;

use actix_cors::Cors;
use actix_web::get;
use actix_web::web::ReqData;
use actix_web::{http::StatusCode, middleware, web, App, HttpResponse, HttpServer, Result};
use actix_web_lab::middleware::ErrorHandlers;
use authentication::middleware::api_key_auth_middleware;

use crate::error::{
    bad_request_handler, internal_server_error_handler, not_found_handler, unauthorized_handler,
};

// CORS configuration
fn cors() -> Cors {
    match std::env::var("ENV")
        .unwrap_or_else(|_| "dev".to_string())
        .as_str()
    {
        "prod" => Cors::default()
            .allow_any_method()
            .allow_any_header()
            .expose_any_header()
            .allowed_origin("https://car-booking.app")
            .supports_credentials(),
        _ => Cors::default()
            .allow_any_method()
            .allow_any_header()
            .allow_any_origin()
            .expose_any_header()
            .supports_credentials(),
    }
}

// API endpoints
#[get("/identity")]
async fn get_identity(
    identity: ReqData<crate::authentication::identity::Identity>,
) -> Result<HttpResponse, actix_web::Error> {
    Ok(HttpResponse::Ok().json(identity.into_inner()))
}

#[get("/health/mongodb")]
async fn mongodb_health() -> Result<HttpResponse, actix_web::Error> {
    match services::mongodb::get_database("vehicle_booking").await {
        Ok(db) => {
            // Try to ping the database
            match db.run_command(mongodb::bson::doc! { "ping": 1 }).await {
                Ok(_) => Ok(HttpResponse::Ok().json(serde_json::json!({
                    "status": "healthy",
                    "message": "MongoDB connection is working"
                }))),
                Err(e) => Ok(HttpResponse::ServiceUnavailable().json(serde_json::json!({
                    "status": "unhealthy",
                    "message": format!("MongoDB ping failed: {}", e)
                }))),
            }
        }
        Err(e) => Ok(HttpResponse::ServiceUnavailable().json(serde_json::json!({
            "status": "unhealthy",
            "message": format!("MongoDB client not available: {}", e)
        }))),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();

    // Initialize Sentry
    let _guard = sentry::init(sentry::ClientOptions {
        dsn: Some("https://42044549243351b661ac8d84f3d587a4@o4509360178003968.ingest.de.sentry.io/4509855303663696".parse().unwrap()),
        release: sentry::release_name!(),
        send_default_pii: true,
        max_request_body_size: sentry::MaxRequestBodySize::Medium,
        ..Default::default()
    });

    let port = std::env::var("API_PORT")
        .unwrap_or_else(|_| std::env::var("PORT").unwrap_or_else(|_| String::from("8080")));

    println!("Starting Vehicle Booking API on port {}", port);
    println!("Available API Keys: Admin, CarManager, MotorbikeManager, Customer1, Customer2");

    HttpServer::new(move || {
        App::new()
            .wrap(cors())
            .wrap(middleware::Logger::new(
                "%{r}a %r %s %b %{Referer}i %{User-Agent}i %T",
            ))
            .wrap(sentry_actix::Sentry::new())
            .wrap(
                ErrorHandlers::new()
                    .handler(StatusCode::BAD_REQUEST, bad_request_handler)
                    .handler(StatusCode::UNAUTHORIZED, unauthorized_handler)
                    .handler(StatusCode::NOT_FOUND, not_found_handler)
                    .handler(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        internal_server_error_handler,
                    ),
            )
            .wrap(middleware::Compress::default()) // Error handlers are now before compression
            .route(
                "/",
                web::get().to(|| async { HttpResponse::Ok().json("Vehicle Booking API") }),
            )
            .service(mongodb_health)
            .service(
                web::scope("/protected")
                    .wrap(middleware::from_fn(api_key_auth_middleware))
                    .service(get_identity)
                    .configure(routes::vehicle::configure)
                    .configure(routes::booking::configure),
            )
    })
    .bind(format!("0.0.0.0:{}", port))?
    .run()
    .await
}
