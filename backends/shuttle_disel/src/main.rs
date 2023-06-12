mod disel_service;
mod models;
mod schema;

use axum::{routing::get, Router};
use disel_service::PgPool;

async fn hello_world() -> &'static str {
    "Hello, world!"
}

#[shuttle_runtime::main]
async fn axum(#[disel_service::Postgres] pool: PgPool) -> shuttle_axum::ShuttleAxum {
    let router = Router::new().route("/", get(hello_world));

    Ok(router.into())
}
