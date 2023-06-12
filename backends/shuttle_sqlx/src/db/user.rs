use jsonwebtoken::DecodingKey;
use serde::Serialize;
use sqlx::PgPool;

use crate::{
    auth::{self},
    error::{AppError, AppResult, DBError},
};

use super::{UserAuth, UserId};

#[derive(Debug, Default, Serialize, sqlx::Type)]
pub struct UserProfile {
    #[serde(skip)]
    pub id: UserId,
    pub username: Option<String>, // This is non-null. Workaround for deriving sqlx::Type.
    pub bio: Option<String>,
    pub image: Option<String>,
    pub following: bool,
}

pub async fn auth_user(pool: &PgPool, token: &str, key: &DecodingKey) -> AppResult<UserAuth> {
    let user_id = auth::verify_token(token, key)?;
    let mut user = get_user(user_id, pool).await?;
    user.token = Some(token.to_string());
    Ok(user)
}

pub async fn get_user(user_id: UserId, pool: &PgPool) -> AppResult<UserAuth> {
    let mut conn = pool.acquire().await.unwrap();

    let user = sqlx::query_as!(
        UserAuth,
        "SELECT *, NULL AS token FROM users WHERE id = $1",
        user_id
    )
    .fetch_optional(&mut conn)
    .await?;

    user.ok_or(AppError::DBError(DBError::NotFound))
}

pub async fn get_user_profile(
    pool: &PgPool,
    username: &str,
    req_user_id: Option<UserId>,
) -> AppResult<UserProfile> {
    let user = sqlx::query_file_as!(
        UserProfile,
        "src/sql/user_profile.sql",
        username,
        req_user_id
    )
    .fetch_one(&mut pool.acquire().await.unwrap())
    .await?;

    Ok(user)
}
