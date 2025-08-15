use actix_cors::Cors;
use actix_web::{App, HttpResponse, HttpServer, middleware, web};

fn cors() -> Cors {
    match std::env::var("ENV").expect("ENV is not set").as_str() {
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

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    let port = std::env::var("PORT").unwrap_or_else(|_| String::from("8080"));

    HttpServer::new(move || {
        App::new()
            .wrap(cors())
            .wrap(middleware::Logger::new(
                "%{r}a %r %s %b %{Referer}i %{User-Agent}i %T",
            ))
            //Middleware for compressing response payloads.
            .wrap(middleware::Compress::default())
            .route("/", web::get().to(HttpResponse::Ok))
    })
    .bind(format!("0.0.0.0:{}", port))?
    .run()
    .await
}
