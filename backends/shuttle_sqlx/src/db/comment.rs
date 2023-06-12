use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;

use super::UserProfile;

#[derive(Debug, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Comment {
    #[serde(skip)]
    pub id: i32,
    pub body: String,
    pub author: UserProfile,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
