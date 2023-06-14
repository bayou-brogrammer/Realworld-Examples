mod api;
mod db;
mod error;
mod routes;
mod utils;

use axum::extract::FromRef;
use jsonwebtoken::{DecodingKey, EncodingKey};
use shuttle_runtime::CustomError;
use shuttle_secrets::SecretStore;
use sqlx::PgPool;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
}

impl FromRef<AppState> for PgPool {
    fn from_ref(app_state: &AppState) -> PgPool {
        app_state.pool.clone()
    }
}

impl FromRef<AppState> for EncodingKey {
    fn from_ref(app_state: &AppState) -> EncodingKey {
        app_state.encoding_key.clone()
    }
}

impl FromRef<AppState> for DecodingKey {
    fn from_ref(app_state: &AppState) -> DecodingKey {
        app_state.decoding_key.clone()
    }
}

#[shuttle_runtime::main]
async fn axum(
    #[shuttle_secrets::Secrets] secret_store: SecretStore,
    // #[shuttle_static_folder::StaticFolder] static_folder: PathBuf,
    #[shuttle_aws_rds::Postgres] pool: PgPool,
) -> shuttle_axum::ShuttleAxum {
    db::prepare_db(&pool).await.map_err(CustomError::new)?;

    let private_key = secret_store.get("PRIVATE_KEY").unwrap();
    let public_key = secret_store.get("PUBLIC_KEY").unwrap();

    Ok(routes::generate_routes(pool, public_key, private_key))
}
