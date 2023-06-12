use serde::Serialize;
use sqlx::FromRow;

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
