use crate::schema::users;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::Queryable;
use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize, Queryable, Identifiable, Selectable)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub username: String,
    pub password: String,
    pub bio: Option<String>,
    pub image: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = users)]
pub struct NewUser {
    pub username: String,
    pub email: String,
    pub password: String,
    pub bio: Option<String>,
    pub image: Option<String>,
}

#[derive(Debug, AsChangeset)]
#[diesel(table_name = users)]
pub struct UserChange {
    pub bio: Option<String>,
    pub email: Option<String>,
    pub image: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
}
