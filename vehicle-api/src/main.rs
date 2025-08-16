mod authentication;

use actix_cors::Cors;
use actix_web::get;
use actix_web::web::ReqData;
use actix_web::{App, HttpResponse, HttpServer, Result, middleware, web};
use authentication::middleware::api_key_auth_middleware;

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
            .allowed_origin("https://my_petstore.com")
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

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    let port = std::env::var("PORT").unwrap_or_else(|_| String::from("8080"));

    println!("Starting Vehicle Booking API on port {}", port);
    println!("Available API Keys: Admin, CarManager, MotorbikeManager, Customer1, Customer2");

    HttpServer::new(move || {
        App::new()
            .wrap(cors())
            .wrap(middleware::Logger::new(
                "%{r}a %r %s %b %{Referer}i %{User-Agent}i %T",
            ))
            .wrap(middleware::Compress::default())
            .route(
                "/",
                web::get().to(|| async { HttpResponse::Ok().json("Vehicle Booking API") }),
            )
            .service(
                web::scope("")
                    .wrap(middleware::from_fn(api_key_auth_middleware))
                    .service(get_identity),
            )
    })
    .bind(format!("0.0.0.0:{}", port))?
    .run()
    .await
}
