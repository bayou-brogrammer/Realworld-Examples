use jsonwebtoken::{DecodingKey, EncodingKey, Header, TokenData, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    error::{AppError, AppResult},
    models::user::User,
};

fn get_encoding_key() -> EncodingKey {
    EncodingKey::from_secret(std::env::var("PUBLIC_KEY").unwrap().as_bytes())
}

fn get_decoding_key() -> DecodingKey {
    DecodingKey::from_secret(std::env::var("PUBLIC_KEY").unwrap().as_bytes())
}

#[derive(Debug)]
pub struct JWTToken(pub String);

// ================================== Claims ================================== //

pub trait GenerateJwt {
    fn generate_jwt(&self) -> Result<String, AppError>;
}

pub trait DecodeJwt {
    fn decode_jwt(&self) -> AppResult<TokenData<Claims>>;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub id: Uuid,
    pub exp: i64,
}

impl GenerateJwt for User {
    fn generate_jwt(&self) -> Result<String, AppError> {
        let exp = (chrono::Utc::now() + chrono::Duration::days(30)).timestamp();
        let claims = Claims { id: self.id, exp };

        Ok(jsonwebtoken::encode(
            &Header::default(),
            &claims,
            &get_encoding_key(),
        )?)
    }
}

impl DecodeJwt for String {
    fn decode_jwt(&self) -> AppResult<TokenData<Claims>> {
        let header = jsonwebtoken::decode_header(self)?;
        match jsonwebtoken::decode::<Claims>(
            self,
            &get_decoding_key(),
            &Validation::new(header.alg),
        ) {
            Ok(res) => Ok(res),
            Err(e) => Err(e.into()),
        }
    }
}
