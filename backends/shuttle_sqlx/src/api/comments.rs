use axum::{
    extract::{Path, State},
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
    auth::{self, JWTToken},
    error::{AppError, AppResult},
};

use crate::db::Comment;
use crate::db::UserProfile;

#[derive(Deserialize)]
pub struct AddComment {
    comment: AddCommentData,
}

#[derive(Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
struct AddCommentData {
    #[validate(length(min = 1, message = "body can't be blank"))]
    body: String,
}

// POST /api/articles/:slug/comments
pub async fn create_comment(
    State(pool): State<PgPool>,
    State(key): State<DecodingKey>,
    Path(slug): Path<String>,
    token: Option<TypedHeader<Authorization<JWTToken>>>,
    Json(AddComment { comment }): Json<AddComment>,
) -> AppResult<impl IntoResponse> {
    let Some(TypedHeader(Authorization(token))) = token else{
        return Err(AppError::Unauthorized);
    };

    let user_id = auth::verify_token(&token.0, &key)?;

    let comment = sqlx::query_as!(
        Comment,
        r#"
        WITH comment AS (
            INSERT INTO comments (body, article_id, author_id)
            VALUES ($1, (SELECT id FROM articles WHERE slug = $2), $3)
            RETURNING *
        )
        SELECT
            comment.id,
            comment.created_at,
            comment.updated_at,
            comment.body,
            (
                users.id,
                users.username,
                users.bio,
                users.image,
                EXISTS (
                    SELECT 1
                    FROM follows
                    WHERE follows.follower_id = $3
                        AND follows.followee_id = users.id
                )
            ) AS "author!: UserProfile"
        FROM comment INNER JOIN users ON users.id = comment.author_id
        "#,
        comment.body,
        slug,
        user_id,
    )
    .fetch_one(&pool)
    .await?;

    Ok(Json(json!({ "comment": comment })))
}

// GET /api/articles/:slug/comments
pub async fn get_comments(
    State(pool): State<PgPool>,
    State(key): State<DecodingKey>,
    Path(slug): Path<String>,
    token: Option<TypedHeader<Authorization<JWTToken>>>,
) -> AppResult<impl IntoResponse> {
    let user_id = token
        .map(|TypedHeader(Authorization(token))| auth::verify_token(&token.0, &key))
        .transpose()?;

    let comments = sqlx::query_as!(
        Comment,
        r#"
        SELECT
            comments.id,
            comments.created_at,
            comments.updated_at,
            comments.body,
            (
                users.id,
                users.username,
                users.bio,
                users.image,
                EXISTS (
                    SELECT 1
                    FROM follows
                    WHERE follows.follower_id = $2
                        AND follows.followee_id = users.id
                )
            ) AS "author!: UserProfile"
        FROM comments 
        INNER JOIN users ON users.id = comments.author_id
        WHERE comments.article_id = (SELECT id FROM articles WHERE slug = $1)
        ORDER BY comments.created_at DESC
        "#,
        slug,
        user_id,
    )
    .fetch_all(&pool)
    .await?;

    Ok(Json(json!({ "comments": comments })))
}

// DELETE /api/articles/:slug/comments/:id
pub async fn delete_comment(
    State(pool): State<PgPool>,
    State(key): State<DecodingKey>,
    Path((slug, comment_id)): Path<(String, i32)>,
    token: Option<TypedHeader<Authorization<JWTToken>>>,
) -> AppResult<impl IntoResponse> {
    let Some(TypedHeader(Authorization(token))) = token else{
        return Err(AppError::Unauthorized);
    };

    let user_id = auth::verify_token(&token.0, &key)?;

    let deleted = sqlx::query!(
        r#"
        DELETE FROM comments
        WHERE id = $1
            AND author_id = $2
            AND article_id = (SELECT id FROM articles WHERE slug = $3)
        RETURNING *
        "#,
        comment_id,
        user_id,
        slug,
    )
    .fetch_all(&pool)
    .await?;

    match deleted.len() {
        0 => Ok(Json(json!({ "message": "Comment not found.", "code": 1 }))),
        _ => Ok(Json(json!({ "message": "OK" }))),
    }
}
