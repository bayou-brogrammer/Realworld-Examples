use crate::{
    db::GenerateAuth,
    error::{AppError, AppResult},
    models::user::User,
    AppState,
};

use actix_web::{
    http::header::{HeaderValue, AUTHORIZATION},
    web::Data,
    HttpRequest, Result,
};

const SCHEME: &str = "Token";

// expand this as needed
#[derive(Debug, Clone)]
pub struct Auth {
    pub user: User,
    pub token: String,
}

pub async fn authenticate(state: &Data<AppState>, req: &HttpRequest) -> Result<Auth, AppError> {
    let db = state.db.clone();

    let token = preprocess_authz_token(req.headers().get(AUTHORIZATION))?;
    db.send(GenerateAuth { token }).await?
}

fn preprocess_authz_token(token: Option<&HeaderValue>) -> AppResult<String> {
    let mut it = match token {
        Some(token) => token.to_str().ok().unwrap().split_whitespace(),
        None => return Err(AppError::Unauthorized("No authorization was provided")),
    };

    let scheme = it.next().unwrap();
    let token = it.next().unwrap();

    if scheme != SCHEME {
        return Err(AppError::Unauthorized("Invalid authorization method"));
    }

    Ok(token.to_string())
}
