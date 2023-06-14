use axum::{
    extract::{Path, Query, State},
    headers::Authorization,
    response::IntoResponse,
    Json, TypedHeader,
};
use jsonwebtoken::DecodingKey;
use serde::Deserialize;
use serde_json::json;
use sqlx::PgPool;
use validator::Validate;

use crate::{
    db::{retrieve_article, Article, UserProfile},
    error::{AppError, AppResult, DBError},
    utils::jwt::{self, JWTToken},
};

#[derive(Deserialize)]
pub struct CreateArticle {
    article: CreateArticleData,
}

#[derive(Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
struct CreateArticleData {
    #[serde(default)]
    tag_list: Vec<String>,
    #[validate(length(min = 1, message = "title can't be blank"))]
    title: String,
    #[validate(length(min = 1, message = "description can't be blank"))]
    description: String,
    #[validate(length(min = 1, message = "body can't be blank"))]
    body: String,
}

// GET /api/articles
pub async fn create_article(
    State(pool): State<PgPool>,
    State(key): State<DecodingKey>,
    token: Option<TypedHeader<Authorization<JWTToken>>>,
    Json(CreateArticle { article }): Json<CreateArticle>,
) -> AppResult<impl IntoResponse> {
    let Some(TypedHeader(Authorization(token))) = token else{
        return Err(AppError::Unauthorized);
    };

    article.validate()?;

    let user_id = jwt::verify_token(&token.0, &key)?;

    let slug = slug::slugify(&article.title);
    let tags = article.tag_list;

    let article = sqlx::query_file_as!(
        Article,
        "src/sql/articles/create_article.sql",
        slug,
        article.title,
        article.description,
        article.body,
        user_id
    )
    .fetch_one(&mut pool.acquire().await.unwrap())
    .await;

    let mut article = match article {
        Ok(article) => article,
        Err(_) => return Err(DBError::ArticleAlreadyCreated.into()),
    };

    sqlx::query!(
        "
        INSERT INTO tags (name)
        SELECT * FROM UNNEST($1::TEXT[])
        ON CONFLICT DO NOTHING
        ",
        &tags[..]
    )
    .execute(&mut pool.acquire().await.unwrap())
    .await?;

    sqlx::query!(
        "
        INSERT INTO article_tags (article_id, tag_id)
        SELECT $1, tags.id FROM tags WHERE tags.name = ANY($2)
        ",
        article.id,
        &tags[..],
    )
    .execute(&mut pool.acquire().await.unwrap())
    .await?;

    article.tag_list = tags;

    Ok(Json(json!({ "article": article })))
}

#[derive(Debug, Deserialize)]
pub struct ListArticlesQuery {
    #[serde(default)]
    tag: Option<String>,
    #[serde(default)]
    author: Option<String>,
    #[serde(default)]
    favorited: Option<String>,
    #[serde(default)]
    limit: Option<usize>,
    #[serde(default)]
    offset: Option<usize>,
}

// GET /api/articles
pub async fn get_articles(
    State(pool): State<PgPool>,
    State(key): State<DecodingKey>,
    Query(params): Query<ListArticlesQuery>,
    token: Option<TypedHeader<Authorization<JWTToken>>>,
) -> AppResult<impl IntoResponse> {
    let user_id = token
        .map(|TypedHeader(Authorization(token))| jwt::verify_token(&token.0, &key))
        .transpose()?;

    let articles = sqlx::query_file_as!(
        Article,
        "src/sql/articles/list_articles.sql",
        params.author,
        params.favorited,
        params.tag,
        params.limit.unwrap_or(20) as i64,
        params.offset.unwrap_or(0) as i64,
        user_id,
    )
    .fetch_all(&mut pool.acquire().await.unwrap())
    .await?;

    let count = articles.len();
    Ok(Json(
        json!({ "articles": articles, "articlesCount": count }),
    ))
}

#[derive(Deserialize)]
pub struct ListFeedArticlesQuery {
    #[serde(default)]
    limit: Option<usize>,
    #[serde(default)]
    offset: Option<usize>,
}

// /api/articles/feed
pub async fn get_feed_articles(
    State(pool): State<PgPool>,
    State(key): State<DecodingKey>,
    Query(params): Query<ListFeedArticlesQuery>,
    token: Option<TypedHeader<Authorization<JWTToken>>>,
) -> AppResult<impl IntoResponse> {
    let Some(TypedHeader(Authorization(token))) = token else{
        return Err(AppError::Unauthorized);
    };

    let user_id = jwt::verify_token(&token.0, &key)?;

    let articles = sqlx::query_file_as!(
        Article,
        "src/sql/articles/feed_articles.sql",
        user_id,
        params.limit.unwrap_or(20) as i64,
        params.offset.unwrap_or(0) as i64,
    )
    .fetch_all(&mut pool.acquire().await.unwrap())
    .await?;

    let count = articles.len();
    Ok(Json(
        json!({ "articles": articles, "articlesCount": count }),
    ))
}

// GET /api/articles/:slug
pub async fn get_article(
    State(pool): State<PgPool>,
    State(key): State<DecodingKey>,
    Path(slug): Path<String>,
    token: Option<TypedHeader<Authorization<JWTToken>>>,
) -> AppResult<impl IntoResponse> {
    let user_id = token
        .map(|TypedHeader(Authorization(token))| jwt::verify_token(&token.0, &key))
        .transpose()?;

    let article = retrieve_article(&pool, slug, user_id).await?;
    Ok(Json(json!({ "article": article })))
}

// /api/articles/:slug
pub async fn delete_article(
    State(pool): State<PgPool>,
    State(key): State<DecodingKey>,
    Path(slug): Path<String>,
    token: Option<TypedHeader<Authorization<JWTToken>>>,
) -> AppResult<impl IntoResponse> {
    let Some(TypedHeader(Authorization(token))) = token else{
        return Err(AppError::Unauthorized);
    };

    let user_id = jwt::verify_token(&token.0, &key)?;
    let deleted = sqlx::query!(
        r#"
        DELETE FROM articles
        WHERE slug = $1 AND author_id = $2
        RETURNING *
        "#,
        slug,
        user_id,
    )
    .fetch_all(&mut pool.acquire().await.unwrap())
    .await?;

    match deleted.len() {
        0 => Ok(Json(json!({ "message": "Article not found.", "code": 1 }))),
        _ => Ok(Json(json!({ "message": "OK" }))),
    }
}

#[derive(Debug, Deserialize)]
pub struct UpdateArticleData {
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    body: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
pub struct UpdateArticle {
    article: UpdateArticleData,
}

pub async fn update_article(
    State(pool): State<PgPool>,
    State(key): State<DecodingKey>,
    Path(slug): Path<String>,
    token: Option<TypedHeader<Authorization<JWTToken>>>,
    Json(UpdateArticle { article }): Json<UpdateArticle>,
) -> AppResult<impl IntoResponse> {
    let Some(TypedHeader(Authorization(token))) = token else{
        return Err(AppError::Unauthorized);
    };

    let user_id = jwt::verify_token(&token.0, &key)?;
    let new_slug = article.title.as_ref().map(slug::slugify);

    let article = sqlx::query_file_as!(
        Article,
        "src/sql/articles/update_article.sql",
        article.title,
        article.description,
        article.body,
        new_slug,
        slug,
        user_id,
    )
    .fetch_one(&mut pool.acquire().await.unwrap())
    .await?;

    Ok(Json(json!({ "article": article })))
}

// POST /api/articles/:slug/favorite
pub async fn favorite_article(
    State(pool): State<PgPool>,
    State(key): State<DecodingKey>,
    Path(slug): Path<String>,
    token: Option<TypedHeader<Authorization<JWTToken>>>,
) -> AppResult<impl IntoResponse> {
    let Some(TypedHeader(Authorization(token))) = token else{
        return Err(AppError::Unauthorized);
    };

    let user_id = jwt::verify_token(&token.0, &key)?;

    sqlx::query!(
        "
        INSERT INTO article_favs (article_id, user_id)
        SELECT articles.id, $2
            FROM articles
            WHERE articles.slug = $1
        ",
        slug,
        user_id
    )
    .execute(&mut pool.acquire().await.unwrap())
    .await?;

    let article = retrieve_article(&pool, slug, Some(user_id)).await?;
    Ok(Json(json!({ "article": article })))
}

// DELETE /api/articles/:slug/favorite
pub async fn un_favorite(
    State(pool): State<PgPool>,
    State(key): State<DecodingKey>,
    Path(slug): Path<String>,
    token: Option<TypedHeader<Authorization<JWTToken>>>,
) -> AppResult<impl IntoResponse> {
    let Some(TypedHeader(Authorization(token))) = token else{
        return Err(AppError::Unauthorized);
    };

    let user_id = jwt::verify_token(&token.0, &key)?;

    sqlx::query!(
        "
        DELETE FROM article_favs
        WHERE article_id = (
            SELECT articles.id
            FROM articles
            WHERE articles.slug = $1
        ) AND article_favs.user_id = $2
        ",
        slug,
        user_id
    )
    .execute(&mut pool.acquire().await.unwrap())
    .await?;

    let article = retrieve_article(&pool, slug, Some(user_id)).await?;
    Ok(Json(json!({ "article": article })))
}
