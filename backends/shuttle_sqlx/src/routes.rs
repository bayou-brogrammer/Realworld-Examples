use axum::{
    error_handling::HandleErrorLayer,
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post, put},
    BoxError, Router,
};
use jsonwebtoken::{DecodingKey, EncodingKey};
use shuttle_axum::AxumService;
use sqlx::PgPool;
use std::time::Duration;
use tower::{buffer::BufferLayer, limit::RateLimitLayer, ServiceBuilder};
use tower_http::compression::CompressionLayer;

use crate::{api, db, AppState};

pub fn generate_routes(pool: PgPool, public_key: String, private_key: String) -> AxumService {
    let encoding_key = EncodingKey::from_rsa_pem(private_key.as_bytes()).unwrap();
    let decoding_key = DecodingKey::from_rsa_pem(public_key.as_bytes()).unwrap();

    let state = AppState {
        pool,
        encoding_key,
        decoding_key,
    };

    Router::new()
        // ==== USERS ==== //
        .route("/api/users/login", post(api::auth::login)) // login
        .route("/api/users", post(api::auth::registration)) // register
        .route("/api/user", get(api::user::get_current_user)) // get user
        .route("/api/user", put(api::user::update_user)) // update user
        // ==== PROFILES ==== //
        .route("/api/profiles/:username", get(api::user::get_profile))
        .route(
            "/api/profiles/:username/follow",
            post(api::user::follow_profile),
        )
        .route(
            "/api/profiles/:username/follow",
            delete(api::user::unfollow_profile),
        )
        // ==== ARTICLES ==== //
        .route("/api/articles", post(api::articles::create_article)) // create article
        .route("/api/articles", get(api::articles::get_articles)) // get articles
        .route("/api/articles/:slug", get(api::articles::get_article)) // get articles
        .route("/api/articles/feed", get(api::articles::get_feed_articles)) // get articles feed
        .route("/api/articles/:slug", put(api::articles::update_article)) // update article
        .route("/api/articles/:slug", delete(api::articles::delete_article)) // delete article
        .route(
            "/api/articles/:slug/favorite",
            post(api::articles::favorite_article),
        )
        .route(
            "/api/articles/:slug/favorite",
            delete(api::articles::un_favorite),
        )
        // ==== COMMENTS ==== //
        // create comment
        .route(
            "/api/articles/:slug/comments",
            get(api::comments::get_comments),
        )
        // create comment
        .route(
            "/api/articles/:slug/comments",
            post(api::comments::create_comment),
        )
        // delete comments
        .route(
            "/api/articles/:slug/comments/:id",
            delete(api::comments::delete_comment),
        )
        // ==== TAGS ==== //
        .route("/api/tags", get(api::tags::get_tags)) // get tags
        // ==== DB ==== //
        .route("/api/initialize", post(db::initialize))
        .fallback(handler_404)
        .with_state(state)
        .layer(CompressionLayer::new())
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(|err: BoxError| async move {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Unhandled error: {}", err),
                    )
                }))
                .layer(BufferLayer::new(1024))
                .layer(RateLimitLayer::new(5, Duration::from_secs(1))),
        )
        .into()
}

async fn handler_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "nothing to see here")
}
