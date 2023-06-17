use chrono::NaiveDateTime;
use diesel::{AsChangeset, Identifiable, Insertable, Queryable, Selectable};
use serde::Deserialize;
use uuid::Uuid;

use crate::schema::{articles, favorite_articles};

#[derive(Debug, Clone, Deserialize, Queryable, Identifiable, Selectable, Insertable)]
pub struct Article {
    pub id: Uuid,
    pub body: String,
    pub description: String,
    pub title: String,
    pub slug: String,
    pub author_id: Uuid,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Deserialize, Insertable)]
#[diesel(table_name = articles)]
pub struct NewArticle {
    pub slug: String,
    pub body: String,
    pub title: String,
    pub author_id: Uuid,
    pub description: String,
}

#[derive(Debug, Clone, Deserialize, AsChangeset)]
#[diesel(table_name = articles)]
pub struct UpdateArticle {
    pub slug: Option<String>,
    pub body: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Insertable)]
#[diesel(table_name = favorite_articles)]
pub struct NewFavoriteArticle {
    pub user_id: Uuid,
    pub article_id: Uuid,
}
