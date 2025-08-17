use actix_web::{
    body::MessageBody,
    dev::{ServiceRequest, ServiceResponse},
    error::ErrorUnauthorized,
    middleware, Error, HttpMessage, HttpRequest, Result,
};
use actix_web_grants::authorities::AttachAuthorities;

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
            // Handle customer API keys and role mapping
            let (role, user_id) = match key.as_str() {
                "Admin" => (super::identity::Role::Admin, "Admin".to_string()),
                "CarManager" => (super::identity::Role::CarManager, "CarManager".to_string()),
                "MotorbikeManager" => (
                    super::identity::Role::MotorbikeManager,
                    "MotorbikeManager".to_string(),
                ),
                // Customer API keys map to Customer role but different user_ids
                "Customer1" => (
                    super::identity::Role::Customer,
                    "customer_user_1".to_string(),
                ),
                "Customer2" => (
                    super::identity::Role::Customer,
                    "customer_user_2".to_string(),
                ),
                _ => return Err(ErrorUnauthorized("Invalid API key")),
            };

            let identity = super::identity::Identity {
                role: role.clone(),
                user_id,
            };

            // Capture identity to Sentry using breadcrumbs and user context
            sentry::configure_scope(|scope| {
                // Set user context for the entire scope
                scope.set_user(Some(sentry::User {
                    id: Some(identity.user_id.clone()),
                    username: Some(identity.user_id.clone()),
                    email: None,
                    ip_address: None,
                    other: {
                        let mut map = std::collections::BTreeMap::new();
                        map.insert(
                            "role".to_string(),
                            sentry::protocol::Value::String(identity.role.to_string()),
                        );
                        map.insert(
                            "api_key".to_string(),
                            sentry::protocol::Value::String(key.clone()),
                        );
                        map
                    },
                }));
                scope.set_tag("user_role", &identity.role.to_string());
                scope.set_tag("user_id", &identity.user_id);
            });

            // Add breadcrumb for authentication event
            sentry::add_breadcrumb(sentry::Breadcrumb {
                ty: "auth".to_string(),
                category: Some("authentication".to_string()),
                message: Some(format!(
                    "User authenticated: {} with role {}",
                    identity.user_id, identity.role
                )),
                data: {
                    let mut map = std::collections::BTreeMap::new();
                    map.insert(
                        "user_id".to_string(),
                        sentry::protocol::Value::String(identity.user_id.clone()),
                    );
                    map.insert(
                        "role".to_string(),
                        sentry::protocol::Value::String(identity.role.to_string()),
                    );
                    map.insert(
                        "api_key".to_string(),
                        sentry::protocol::Value::String(key.clone()),
                    );
                    map.insert(
                        "method".to_string(),
                        sentry::protocol::Value::String(req.method().to_string()),
                    );
                    map.insert(
                        "path".to_string(),
                        sentry::protocol::Value::String(req.path().to_string()),
                    );
                    map.insert(
                        "timestamp".to_string(),
                        sentry::protocol::Value::String(chrono::Utc::now().to_rfc3339()),
                    );
                    map
                },
                level: sentry::Level::Info,
                timestamp: std::time::SystemTime::now(),
            });

            // Attach authorities (roles) for actix-web-grants
            req.attach(vec![role.clone()]);

            // Attach role and identity to request extensions
            req.extensions_mut().insert(identity);

            // Continue to next middleware/handler
            next.call(req).await
        }
        None => Err(ErrorUnauthorized("Missing X-API-Key header")),
    }
}
