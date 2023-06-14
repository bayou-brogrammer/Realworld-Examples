use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header};
use serde::Serialize;
use sqlx::FromRow;

use crate::{
    error::{AppError, AppResult},
    utils::jwt::Claims,
};

pub type UserId = i32;

#[derive(Debug, Default, Serialize, FromRow)]
pub struct UserAuth {
    #[serde(skip)]
    pub id: UserId,
    #[serde(skip)]
    pub hash: String,
    pub email: String,
    pub username: String,
    pub bio: Option<String>,
    pub image: Option<String>,
    pub token: Option<String>,
}

impl UserAuth {
    pub fn generate_jwt(&mut self, key: &EncodingKey) -> AppResult<()> {
        let exp = (chrono::Utc::now() + chrono::Duration::days(30)).timestamp();
        let claims = Claims {
            exp,
            user_id: self.id,
        };
        let token = jsonwebtoken::encode(&Header::new(Algorithm::RS384), &claims, key)?;
        self.token = Some(token);
        Ok(())
    }

    pub fn decode_jwt(&self, key: &DecodingKey) -> AppResult<Claims> {
        let Some(token) = self.token.as_ref() else{
            return Err(AppError::Unauthorized)
        };

        jsonwebtoken::decode_header(token)
            .map(|header| {
                jsonwebtoken::decode::<Claims>(
                    token,
                    key,
                    &jsonwebtoken::Validation::new(header.alg),
                )
            })
            .map(|data| match data {
                Ok(data) => Ok(data.claims),
                Err(err) => Err(err.into()),
            })?
    }
}
