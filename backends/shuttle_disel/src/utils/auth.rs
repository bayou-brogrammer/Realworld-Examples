use crate::{
    db::GenerateAuth,
    error::{AppError, AppResult},
    models::user::User,
    AppState,
};
use actix_web::{
    http::header::{HeaderValue, AUTHORIZATION},
    web::{self, Data},
    HttpRequest,
};
use futures::{future::ready, Future};
use serde::Deserialize;
use std::pin::Pin;

const SCHEME: &str = "Token";

// expand this as needed
#[derive(Debug, Clone, Deserialize)]
pub struct Auth {
    pub user: User,
    pub token: String,
}

pub async fn authenticate(state: &Data<AppState>, req: &HttpRequest) -> AppResult<Auth> {
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

// =================== EXTRACTOR =================== //

#[derive(Debug)]
pub struct AuthExtractor(pub AppResult<Auth>);

impl std::ops::Deref for AuthExtractor {
    type Target = AppResult<Auth>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AuthExtractor {
    fn auth(auth: Auth) -> AppResult<Self> {
        Ok(AuthExtractor(Ok(auth)))
    }

    fn error(error: AppError) -> AppResult<Self> {
        Ok(AuthExtractor(Err(error)))
    }
}

impl actix_web::FromRequest for AuthExtractor {
    type Error = AppError;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(req: &HttpRequest, _: &mut actix_web::dev::Payload) -> Self::Future {
        let state = req.app_data::<web::Data<AppState>>().unwrap();
        let db = state.db.clone();

        let token = match preprocess_authz_token(req.headers().get(AUTHORIZATION)) {
            Ok(token) => token,
            Err(e) => return Box::pin(ready(AuthExtractor::error(e))),
        };

        Box::pin(async move {
            match db.send(GenerateAuth { token }).await? {
                Err(e) => AuthExtractor::error(e),
                Ok(auth) => AuthExtractor::auth(auth),
            }
        })
    }
}
