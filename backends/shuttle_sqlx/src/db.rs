mod user;
pub use user::*;
mod article;
pub use article::*;
mod comment;
pub use comment::*;
mod tag;
pub use tag::*;

use axum::{extract::State, response::IntoResponse, Json};
use serde_json::json;
use sqlx::{Executor, PgPool};

use crate::error::AppResult;

pub async fn prepare_db(pool: &PgPool) -> Result<(), sqlx::Error> {
    pool.execute(include_str!("sql/schema.sql")).await?;
    Ok(())
}

#[allow(dead_code)]
pub async fn initialize_db(pool: &PgPool) -> Result<(), sqlx::Error> {
    pool.execute(include_str!("sql/down.sql")).await?;
    pool.execute(include_str!("sql/schema.sql")).await?;
    Ok(())
}

#[allow(dead_code)]
pub async fn initialize(State(pool): State<PgPool>) -> AppResult<impl IntoResponse> {
    initialize_db(&pool).await?;
    Ok(Json(json!({ "message": "ok" })))
}
