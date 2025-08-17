use actix_web::{body, dev, http::StatusCode};
use actix_web::{HttpResponse, ResponseError};
use derive_more::Display;
use serde::Serialize;

#[allow(dead_code)]
#[derive(Debug, Display, Serialize)]
pub enum AppError {
    #[display("Not found: {}", message)]
    NotFound { message: String },
    #[display("Forbidden: {}", message)]
    Forbidden { message: String },
    #[display("Unauthorized: {}", message)]
    Unauthorized { message: String },
    #[display("Internal server error: {}", message)]
    InternalServerError { message: String },
    #[display("Invalid request parameters: {}", message)]
    BadRequest { message: String },
}

pub type AppResult<T> = std::result::Result<T, AppError>;

#[macro_export]
macro_rules! internal_error {
    ($target:ty : $($other:path), *) => {
        $(
            impl From<$other> for $target {
                fn from(other: $other) -> Self {
                    Self::InternalServerError { message: other.to_string() }
                }
            }
        )*
    }
}

internal_error!(
    AppError: std::io::Error,
    mongodb::error::Error
);

#[derive(Serialize)]
struct ErrorResponse {
    code: u16,
    message: String,
    error_type: String,
}

impl ResponseError for AppError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        match self {
            AppError::NotFound { .. } => actix_web::http::StatusCode::NOT_FOUND,
            AppError::Forbidden { .. } => actix_web::http::StatusCode::FORBIDDEN,
            AppError::Unauthorized { .. } => actix_web::http::StatusCode::UNAUTHORIZED,
            AppError::InternalServerError { .. } => {
                actix_web::http::StatusCode::INTERNAL_SERVER_ERROR
            }
            AppError::BadRequest { .. } => actix_web::http::StatusCode::BAD_REQUEST,
        }
    }

    fn error_response(&self) -> HttpResponse {
        let status_code = self.status_code();
        let error_response = ErrorResponse {
            code: status_code.as_u16(),
            message: self.to_string(),
            error_type: format!("{:?}", self),
        };

        HttpResponse::build(status_code).json(error_response)
    }
}

#[allow(dead_code)]
impl AppError {
    pub fn not_found(message: impl Into<String>) -> Self {
        AppError::NotFound {
            message: message.into(),
        }
    }

    pub fn forbidden(message: impl Into<String>) -> Self {
        AppError::Forbidden {
            message: message.into(),
        }
    }

    pub fn unauthorized(message: impl Into<String>) -> Self {
        AppError::Unauthorized {
            message: message.into(),
        }
    }

    pub fn internal_server_error(message: impl Into<String>) -> Self {
        AppError::InternalServerError {
            message: message.into(),
        }
    }

    pub fn bad_request(message: impl Into<String>) -> Self {
        AppError::BadRequest {
            message: message.into(),
        }
    }
}

async fn generic_error_handler<B>(
    res: dev::ServiceResponse<B>,
    status_code: StatusCode,
    error_type: &str,
    default_message: &str,
) -> Result<dev::ServiceResponse<actix_web::body::EitherBody<B>>, actix_web::Error>
where
    B: actix_web::body::MessageBody + 'static,
{
    let (req, res) = res.into_parts();
    let (_res, body) = res.into_parts();

    let body_bytes = body::to_bytes(body).await.ok().unwrap_or_default();

    let body_string = if body_bytes.is_empty() {
        "".to_string()
    } else {
        String::from_utf8(body_bytes.to_vec())
            .unwrap_or_else(|_| String::from_utf8_lossy(&body_bytes).to_string())
    };

    let message = if body_string.is_empty() {
        default_message.to_string()
    } else {
        let truncated_body = if body_string.len() > 500 {
            format!("{}... (truncated)", &body_string[..500])
        } else {
            body_string
        };
        format!("{}: {}", default_message, truncated_body)
    };

    let error_response = ErrorResponse {
        code: status_code.as_u16(),
        message, // This now contains the captured body content!
        error_type: error_type.to_string(),
    };

    let new_response = HttpResponse::build(status_code)
        .json(error_response)
        .map_into_right_body();

    let service_response = dev::ServiceResponse::new(req, new_response);
    Ok(service_response)
}

pub async fn bad_request_handler<B>(
    res: dev::ServiceResponse<B>,
) -> Result<dev::ServiceResponse<actix_web::body::EitherBody<B>>, actix_web::Error>
where
    B: actix_web::body::MessageBody + 'static,
{
    generic_error_handler(res, StatusCode::BAD_REQUEST, "BadRequest", "Bad Request").await
}

pub async fn unauthorized_handler<B>(
    res: dev::ServiceResponse<B>,
) -> Result<dev::ServiceResponse<actix_web::body::EitherBody<B>>, actix_web::Error>
where
    B: actix_web::body::MessageBody + 'static,
{
    generic_error_handler(
        res,
        StatusCode::UNAUTHORIZED,
        "Unauthorized",
        "Unauthorized",
    )
    .await
}

pub async fn not_found_handler<B>(
    res: dev::ServiceResponse<B>,
) -> Result<dev::ServiceResponse<actix_web::body::EitherBody<B>>, actix_web::Error>
where
    B: actix_web::body::MessageBody + 'static,
{
    generic_error_handler(res, StatusCode::NOT_FOUND, "NotFound", "Not Found").await
}

pub async fn internal_server_error_handler<B>(
    res: dev::ServiceResponse<B>,
) -> Result<dev::ServiceResponse<actix_web::body::EitherBody<B>>, actix_web::Error>
where
    B: actix_web::body::MessageBody + 'static,
{
    generic_error_handler(
        res,
        StatusCode::INTERNAL_SERVER_ERROR,
        "InternalServerError",
        "Internal Server Error",
    )
    .await
}
