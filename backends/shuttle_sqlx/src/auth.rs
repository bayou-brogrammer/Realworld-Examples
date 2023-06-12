use axum::headers::authorization::Credentials;
use jsonwebtoken::{encode, Algorithm, DecodingKey, EncodingKey, Header};
use serde::{Deserialize, Serialize};

use crate::{db::UserId, error::AppResult};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    exp: i64,
    pub user_id: UserId,
}

pub fn verify_token(token: &str, key: &DecodingKey) -> AppResult<UserId> {
    let claim = verify_jwt(token, key)?;
    Ok(claim.user_id)
}

pub fn generate_jwt(user_id: UserId, key: &EncodingKey) -> AppResult<String> {
    let exp = (chrono::Utc::now() + chrono::Duration::days(30)).timestamp();
    let claims = Claims { user_id, exp };
    let token = encode(&Header::new(Algorithm::RS384), &claims, key)?;

    Ok(token)
}

pub fn verify_jwt(token: &str, key: &DecodingKey) -> AppResult<Claims> {
    let header = jsonwebtoken::decode_header(token)?;

    let claims =
        jsonwebtoken::decode::<Claims>(token, key, &jsonwebtoken::Validation::new(header.alg))?
            .claims;
    Ok(claims)
}

#[derive(Debug)]
pub struct JWTToken(pub String);

impl Credentials for JWTToken {
    const SCHEME: &'static str = "Token";

    fn decode(value: &axum::http::HeaderValue) -> Option<Self> {
        let mut it = value.to_str().ok()?.split_whitespace();
        let scheme = it.next()?;
        let token = it.next()?;

        if scheme != Self::SCHEME || it.next().is_some() {
            None?
        }

        Some(Self(token.to_string()))
    }

    fn encode(&self) -> axum::http::HeaderValue {
        unreachable!()
    }
}

pub fn hash_password(password: impl AsRef<[u8]>) -> AppResult<String> {
    let salt = password_hash::SaltString::generate(&mut rand::thread_rng());

    let hash =
        password_hash::PasswordHash::generate(argon2::Argon2::default(), password.as_ref(), &salt)
            .map_err(|err| anyhow::anyhow!(err))?
            .to_string();
    Ok(hash)
}
