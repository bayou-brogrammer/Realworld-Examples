use std::future::IntoFuture;

use actix::MailboxError;
use actix_web::{http::StatusCode, HttpResponse, ResponseError};
use diesel::{
    r2d2::PoolError,
    result::{DatabaseErrorKind, Error as DieselError},
};
use jsonwebtoken::errors::{Error as JwtError, ErrorKind as JwtErrorKind};
use libreauth::pass::Error as PassError;
use serde_json::{json, Map};
use thiserror::Error;
use validator::ValidationErrors;

pub type AppResult<T, E = AppError> = std::result::Result<T, E>;

impl IntoFuture for AppError {
    type Output = AppResult<HttpResponse>;
    type IntoFuture = futures::future::Ready<Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        futures::future::ready(Err(self))
    }
}

#[derive(Error, Debug, Clone)]
pub enum AppError {
    // 401
    #[error("Unauthorized")]
    Unauthorized(&'static str),

    // 403
    #[error("Forbidden: {0:?}")]
    Forbidden(&'static str),

    // 404
    #[error("Not Found: {0:?}")]
    NotFound(Option<&'static str>),

    // 422
    #[error("Unprocessable Entity: {0:?}")]
    UnprocessableEntity(serde_json::Value),

    // 500
    #[error("Internal Server Error")]
    InternalServerError,
}

impl AppError {
    pub fn not_found(message: &'static str) -> Self {
        AppError::NotFound(Some(message))
    }
}

// the ResponseError trait lets us convert errors to http responses with appropriate data
// https://actix.rs/docs/errors/
impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        let coerce_error = move |message: &'static str| json!({ "error": message });

        println!("Error: {:?}", self);

        match *self {
            AppError::NotFound(message) => {
                HttpResponse::NotFound().json(coerce_error(message.unwrap_or("Not Found")))
            }
            AppError::Forbidden(message) => HttpResponse::Forbidden().json(coerce_error(message)),
            AppError::UnprocessableEntity(ref message) => {
                HttpResponse::build(StatusCode::UNPROCESSABLE_ENTITY).json(message)
            }
            AppError::Unauthorized(message) => {
                HttpResponse::Unauthorized().json(coerce_error(message))
            }
            _ => HttpResponse::InternalServerError().json("Internal Server Error"),
        }
    }
}

impl From<DieselError> for AppError {
    fn from(error: DieselError) -> Self {
        println!("DiselError: {:?}", error);
        match error {
            DieselError::DatabaseError(kind, info) => {
                if let DatabaseErrorKind::UniqueViolation = kind {
                    let message = info.details().unwrap_or_else(|| info.message()).to_string();
                    return AppError::UnprocessableEntity(json!({ "error": message }));
                }
                AppError::InternalServerError
            }
            DieselError::NotFound => AppError::not_found("Record not found"),
            _ => AppError::InternalServerError,
        }
    }
}

impl From<MailboxError> for AppError {
    fn from(_error: MailboxError) -> Self {
        AppError::InternalServerError
    }
}

impl From<JwtError> for AppError {
    fn from(error: JwtError) -> Self {
        match error.kind() {
            JwtErrorKind::InvalidToken => AppError::Unauthorized("Token is invalid"),
            JwtErrorKind::InvalidIssuer => AppError::Unauthorized("Issuer is invalid"),
            _ => AppError::Unauthorized("An issue was found with the token provided"),
        }
    }
}

impl From<PoolError> for AppError {
    fn from(_error: PoolError) -> Self {
        AppError::InternalServerError
    }
}

impl From<PassError> for AppError {
    fn from(_error: PassError) -> Self {
        AppError::InternalServerError
    }
}

impl From<ValidationErrors> for AppError {
    fn from(errors: ValidationErrors) -> Self {
        let mut err_map = Map::new();

        // transforms errors into objects that err_map can take
        for (field, errors) in errors.field_errors().iter() {
            let errors: Vec<serde_json::Value> = errors
                .iter()
                .map(|error| {
                    dbg!(error); // <- Uncomment this if you want to see what error looks like
                    json!(error.message)
                })
                .collect();
            err_map.insert(field.to_string(), json!(errors));
        }

        AppError::UnprocessableEntity(json!({ "errors": err_map }))
    }
}
