use axum::headers::authorization::Credentials;
use jsonwebtoken::DecodingKey;
use serde::{Deserialize, Serialize};

use crate::error::AppResult;

use super::auth::UserId;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub exp: i64,
    pub user_id: UserId,
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

pub fn verify_token(token: &str, key: &DecodingKey) -> AppResult<UserId> {
    let claim = verify_jwt(token, key)?;
    Ok(claim.user_id)
}

pub fn verify_jwt(token: &str, key: &DecodingKey) -> AppResult<Claims> {
    let header = jsonwebtoken::decode_header(token)?;

    let claims =
        jsonwebtoken::decode::<Claims>(token, key, &jsonwebtoken::Validation::new(header.alg))?
            .claims;
    Ok(claims)
}
