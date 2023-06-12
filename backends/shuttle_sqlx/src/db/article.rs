use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::{FromRow, PgPool};

use crate::error::AppResult;

use super::UserProfile;

#[derive(Debug, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Article {
    #[serde(skip)]
    pub(crate) id: i32,
    pub(crate) slug: String,
    pub(crate) body: String,
    pub(crate) title: String,
    pub(crate) favorited: bool,
    pub(crate) description: String,
    pub(crate) author: UserProfile,
    pub(crate) favorites_count: i64,
    pub(crate) tag_list: Vec<String>,
    pub(crate) created_at: DateTime<Utc>,
    pub(crate) updated_at: DateTime<Utc>,
}

pub async fn retrieve_article(
    pool: &PgPool,
    slug: String,
    user_id: Option<i32>,
) -> AppResult<Article> {
    Ok(
        sqlx::query_file_as!(Article, "src/sql/articles/get_article.sql", slug, user_id,)
            .fetch_one(&mut pool.acquire().await.unwrap())
            .await?,
    )
}
