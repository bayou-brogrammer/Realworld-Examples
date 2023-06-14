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
    db::{auth_user, get_user_profile},
    error::{AppError, AppResult},
    utils::{
        auth::UserAuth,
        hasher,
        jwt::{self, JWTToken},
    },
};

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateUserData {
    pub bio: Option<String>,
    pub image: Option<String>,

    #[validate(email)]
    pub email: Option<String>,
    #[validate(non_control_character, length(min = 1, max = 64))]
    pub username: Option<String>,
    #[validate(non_control_character, length(min = 8, max = 64))]
    pub password: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
pub struct UpdateUser {
    user: UpdateUserData,
}

// GET /api/user
pub async fn get_current_user(
    State(pool): State<PgPool>,
    State(key): State<DecodingKey>,
    TypedHeader(Authorization(token)): TypedHeader<Authorization<JWTToken>>,
) -> AppResult<impl IntoResponse> {
    let user = auth_user(&pool, &token.0, &key).await?;
    Ok(Json(json!({ "user": user })))
}

// PUT /api/user
pub async fn update_user(
    State(pool): State<PgPool>,
    State(key): State<DecodingKey>,
    header: Option<TypedHeader<Authorization<JWTToken>>>,
    Json(UpdateUser { user: updated_user }): Json<UpdateUser>,
) -> AppResult<impl IntoResponse> {
    let Some(TypedHeader(Authorization(token))) = header else{
        return Err(AppError::Unauthorized);
    };

    let user = auth_user(&pool, &token.0, &key).await?;
    let hash = updated_user
        .password
        .map(hasher::hash_password)
        .transpose()?;

    let mut updated_user = sqlx::query_as!(
        UserAuth,
        "UPDATE users
            SET (email, username, bio, image, hash) =
                (
                    COALESCE($1, email),
                    COALESCE($2, username),
                    COALESCE($3, bio),
                    COALESCE($4, image),
                    COALESCE($5, hash)
                )
            WHERE id = $6
        RETURNING *, NULL AS token",
        updated_user.email,
        updated_user.username,
        updated_user.bio,
        updated_user.image,
        hash,
        user.id
    )
    .fetch_one(&mut pool.acquire().await.unwrap())
    .await?;

    updated_user.token = Some(token.0);

    Ok(Json(json!({ "user": updated_user })))
}

// GET /api/user/:username
pub async fn get_profile(
    State(pool): State<PgPool>,
    State(key): State<DecodingKey>,
    Path(username): Path<String>,
    token: Option<TypedHeader<Authorization<JWTToken>>>,
) -> AppResult<impl IntoResponse> {
    let user_id = token
        .map(|TypedHeader(Authorization(token))| jwt::verify_token(&token.0, &key))
        .transpose()?;

    let profile = get_user_profile(&pool, &username, user_id).await?;
    Ok(Json(json!({ "profile": profile })))
}

// POST /api/user/:username/follow
pub async fn follow_profile(
    State(pool): State<PgPool>,
    State(key): State<DecodingKey>,
    Path(username): Path<String>,
    token: Option<TypedHeader<Authorization<JWTToken>>>,
) -> AppResult<impl IntoResponse> {
    let Some(TypedHeader(Authorization(token))) = token else{
        return Err(AppError::Unauthorized);
    };

    let follower_id = jwt::verify_token(&token.0, &key)?;
    let mut followee = get_user_profile(&pool, &username, Some(follower_id)).await?;

    sqlx::query!(
        "
        INSERT INTO follows (follower_id, followee_id)
        VALUES ($1, $2)
        ON CONFLICT DO NOTHING
        ",
        follower_id,
        followee.id
    )
    .execute(&mut pool.acquire().await.unwrap())
    .await?;

    followee.following = true;
    Ok(Json(json!({ "profile": followee })))
}

pub async fn unfollow_profile(
    State(pool): State<PgPool>,
    State(key): State<DecodingKey>,
    Path(username): Path<String>,
    token: Option<TypedHeader<Authorization<JWTToken>>>,
) -> AppResult<impl IntoResponse> {
    let Some(TypedHeader(Authorization(token))) = token else{
        return Err(AppError::Unauthorized);
    };

    let follower_id = jwt::verify_token(&token.0, &key)?;
    let mut followee = get_user_profile(&pool, &username, Some(follower_id)).await?;

    if !followee.following {
        return Ok(Json(json!({ "profile": followee })));
    }

    sqlx::query!(
        "
        DELETE FROM follows
        WHERE follower_id = $1 AND followee_id = $2
        ",
        follower_id,
        followee.id
    )
    .execute(&mut pool.acquire().await.unwrap())
    .await?;

    followee.following = false;
    Ok(Json(json!({ "profile": followee })))
}
