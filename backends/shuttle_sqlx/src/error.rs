use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

pub type AppResult<T> = std::result::Result<T, AppError>;

#[derive(thiserror::Error, Debug)]
pub enum DBError {
    #[error("User is already registered")]
    AlreadyRegistered,

    #[error("Article is already created")]
    ArticleAlreadyCreated,

    #[error("Not Found")]
    NotFound,
}

#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("Any error: {0:?}")]
    Anyhow(#[from] anyhow::Error),

    #[error("DB Error: {0:?}")]
    DBError(#[from] DBError),

    #[error("Forbidden request")]
    Forbidden(&'static str),

    #[error("Unauthorized")]
    Unauthorized,

    #[error("SQL failed: {0:?}")]
    Sqlx(#[from] sqlx::Error),

    #[error("JWT error: {0:?}")]
    JwtError(#[from] jsonwebtoken::errors::Error),

    #[error("Invalid request: {0:?}")]
    Validation(#[from] validator::ValidationErrors),
}

// Tell axum how to convert `AppError` into a response.
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        println!("Error: {:?}", self);

        let (status, error_message) = match self {
            AppError::Forbidden(_) => (StatusCode::FORBIDDEN, None),
            AppError::JwtError(_) => (StatusCode::UNAUTHORIZED, None),
            AppError::Sqlx(_) => (StatusCode::INTERNAL_SERVER_ERROR, None),
            AppError::Anyhow(_) => (StatusCode::INTERNAL_SERVER_ERROR, None),
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, Some(self.to_string())),
            AppError::Validation(_) => (StatusCode::UNPROCESSABLE_ENTITY, Some(self.to_string())),
            AppError::DBError(db_error) => {
                let message = db_error.to_string();

                match db_error {
                    DBError::NotFound => (StatusCode::NOT_FOUND, Some(message)),
                    _ => (StatusCode::OK, Some(message)),
                }
            }
        };

        let body = Json(json!({
            "error": error_message.unwrap_or_else(|| status.canonical_reason().unwrap().to_string()),
        }));

        (status, body).into_response()
    }
}
