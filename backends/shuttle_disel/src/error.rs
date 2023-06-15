use actix_web::{http::StatusCode, HttpResponse, ResponseError};
use diesel::r2d2;
use thiserror::Error;

pub type AppResult<T, E = AppError> = std::result::Result<T, E>;

#[derive(Error, Debug)]
pub enum AppError {
    // 401
    #[error("Unauthorized")]
    Unauthorized(&'static str),

    // 403
    #[error("Forbidden: {0:?}")]
    Forbidden(serde_json::Value),

    // 404
    #[error("Not Found: {0:?}")]
    NotFound(serde_json::Value),

    // 422
    #[error("Unprocessable Entity: {0:?}")]
    UnprocessableEntity(serde_json::Value),

    // 500
    #[error("Internal Server Error")]
    InternalServerError,

    // ======== custom errors ======== //
    #[error("HasherError error: {0:?}")]
    HasherError(#[from] libreauth::pass::Error),

    #[error("JWT error: {0:?}")]
    JwtError(#[from] jsonwebtoken::errors::Error),

    #[error("Disel Pool error: {0:?}")]
    PoolError(#[from] r2d2::PoolError),

    #[error("Disel Connection error: {0:?}")]
    DieselError(#[from] diesel::result::Error),

    #[error("Mailbox error: {0:?}")]
    MailboxError(#[from] actix::MailboxError),

    #[error("Validation error: {0:?}")]
    ValidationError(#[from] validator::ValidationErrors),
}

// the ResponseError trait lets us convert errors to http responses with appropriate data
// https://actix.rs/docs/errors/
impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        match *self {
            AppError::Unauthorized(ref message) => HttpResponse::Unauthorized().json(message),
            AppError::Forbidden(ref message) => HttpResponse::Forbidden().json(message),
            AppError::NotFound(ref message) => HttpResponse::NotFound().json(message),
            AppError::UnprocessableEntity(ref message) => {
                HttpResponse::build(StatusCode::UNPROCESSABLE_ENTITY).json(message)
            }
            _ => HttpResponse::InternalServerError().json("Internal Server Error"),
        }
    }
}
