use crate::schema::users;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::Queryable;
use uuid::Uuid;

#[derive(Debug, Queryable, Identifiable, Clone)]
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
#[table_name = "users"]
pub struct NewUser {
    pub username: String,
    pub email: String,
    pub password: String,
    pub bio: Option<String>,
    pub image: Option<String>,
}
