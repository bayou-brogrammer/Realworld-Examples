mod user;

use axum::{
    error_handling::HandleErrorLayer, http::StatusCode, response::IntoResponse, BoxError, Router,
};
use shuttle_axum::AxumService;
use std::time::Duration;
use tower::{buffer::BufferLayer, limit::RateLimitLayer, ServiceBuilder};
use tower_http::compression::CompressionLayer;

use self::user::user_routes;

async fn handler_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "404 Not Found")
}

pub fn axum_service() -> AxumService {
    Router::new()
        .nest("/api/user", user_routes())
        .fallback(handler_404)
        // .with_state(state)
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
