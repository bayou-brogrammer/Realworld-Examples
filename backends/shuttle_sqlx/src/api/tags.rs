use axum::{extract::State, response::IntoResponse, Json};
use serde_json::json;
use sqlx::PgPool;

use crate::error::AppResult;

use crate::db::Tag;

// GET /api/articles/:slug/comments
pub async fn get_tags(State(pool): State<PgPool>) -> AppResult<impl IntoResponse> {
    let tags = sqlx::query_as!(
        Tag,
        r#"
        SELECT tags.name
        FROM tags
        INNER JOIN article_tags ON article_tags.tag_id = tags.id
        GROUP BY tags.name
        ORDER BY COUNT(article_tags.tag_id) DESC
        "#,
    )
    .fetch_all(&pool)
    .await?;

    let tags = tags
        .into_iter()
        .map(|tag| tag.name)
        .collect::<Vec<String>>();

    Ok(Json(json!({ "tags": tags })))
}
