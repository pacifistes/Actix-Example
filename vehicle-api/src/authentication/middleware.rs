use actix_web::{
    Error, HttpMessage, HttpRequest, Result,
    body::MessageBody,
    dev::{ServiceRequest, ServiceResponse},
    error::ErrorUnauthorized,
    middleware,
};
use actix_web_grants::authorities::AttachAuthorities;
use std::str::FromStr;

// Authentication functions
fn extract_api_key(req: &HttpRequest) -> Option<String> {
    req.headers()
        .get("X-API-Key")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string())
}

// API Key Authentication Middleware using from_fn
pub async fn api_key_auth_middleware(
    req: ServiceRequest,
    next: middleware::Next<impl MessageBody>,
) -> Result<ServiceResponse<impl MessageBody>, Error> {
    // Extract API key from request
    let api_key = extract_api_key(req.request());

    match api_key {
        Some(key) => {
            // Parse role from API key using Strum
            match super::identity::Role::from_str(&key) {
                Ok(role) => {
                    let identity = super::identity::Identity {
                        role: role.clone(),
                        user_id: format!("{:?}", role),
                    };

                    // Attach authorities (roles) for actix-web-grants
                    req.attach(vec![role.clone()]);

                    // Attach role and identity to request extensions
                    req.extensions_mut().insert(identity);

                    // Continue to next middleware/handler
                    next.call(req).await
                }
                Err(_) => Err(ErrorUnauthorized("Invalid API key")),
            }
        }
        None => Err(ErrorUnauthorized("Missing X-API-Key header")),
    }
}
