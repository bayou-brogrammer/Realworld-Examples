use axum::{extract::State, response::IntoResponse, Json};
use jsonwebtoken::EncodingKey;
use serde::Deserialize;
use serde_json::json;
use sqlx::PgPool;
use validator::Validate;

use crate::{
    error::{AppError, AppResult, DBError},
    utils::{auth::UserAuth, hasher},
};

// ================================================= LOGIN ================================================= //

#[derive(Debug, Deserialize)]
pub struct Login {
    user: LoginUser,
}

#[derive(Debug, Deserialize, Validate)]
struct LoginUser {
    #[validate(
        email(message = "invalid email address"),
        length(min = 1, message = "email can't be blank")
    )]
    email: String,
    #[validate(length(min = 1, message = "password can't be blank"))]
    password: String,
}

pub async fn login(
    State(pool): State<PgPool>,
    State(key): State<EncodingKey>,
    Json(Login { user }): Json<Login>,
) -> AppResult<impl IntoResponse> {
    user.validate()?;

    let mut conn = pool.acquire().await.unwrap();

    let user_auth = sqlx::query_as!(
        UserAuth,
        "SELECT *, NULL AS token FROM users WHERE email = $1",
        user.email
    )
    .fetch_optional(&mut conn)
    .await?;

    let Some(mut user_auth) = user_auth else {
        Err(AppError::Forbidden("Invalid User"))?
    };

    let hash =
        password_hash::PasswordHash::new(&user_auth.hash).map_err(|err| anyhow::anyhow!(err))?;

    hash.verify_password(&[&argon2::Argon2::default()], &user.password)
        .map_err(|err| {
            log::error!("err: {:?}", err);
            AppError::Forbidden("email or password is invalid")
        })?;

    user_auth.generate_jwt(&key)?;
    Ok(Json(json!({ "user": user_auth })))
}

// ================================================= REGISTRATION ================================================= //

#[derive(Deserialize, Validate)]
struct RegistrationUser {
    #[validate(
        non_control_character(message = "user name can't contain non-ascii charactors"),
        length(min = 1, message = "user name can't be blank"),
        length(max = 64, message = "too long user name")
    )]
    username: String,

    #[validate(
        length(min = 1, message = "email can't be blank"),
        length(max = 64, message = "too long email address"),
        email(message = "invalid email address")
    )]
    email: String,

    #[validate(
        non_control_character(message = "password can't contain non-ascii charactors"),
        length(min = 8, message = "password must be at least 8 characters long"),
        length(max = 64, message = "too long password")
    )]
    password: String,
}

#[derive(Deserialize)]
pub struct Registration {
    user: RegistrationUser,
}

pub async fn registration(
    State(pool): State<PgPool>,
    State(key): State<EncodingKey>,
    Json(Registration { user }): Json<Registration>,
) -> AppResult<impl IntoResponse> {
    user.validate()?;

    let hash = hasher::hash_password(&user.password)?;
    let mut conn = pool.acquire().await.unwrap();

    let user_auth = sqlx::query_as!(
        UserAuth,
        r#"
        INSERT INTO users (username, email, hash)
        VALUES ($1, $2, $3)
        RETURNING *, NULL AS token
        "#,
        user.username,
        user.email,
        hash
    )
    .fetch_optional(&mut conn)
    .await;

    let user_auth = match user_auth {
        Ok(user_auth) => user_auth,
        Err(_) => return Err(DBError::AlreadyRegistered.into()),
    };

    let mut user_auth = user_auth.unwrap();
    user_auth.generate_jwt(&key)?;
    Ok(Json(json!({ "user": user_auth })))
}
