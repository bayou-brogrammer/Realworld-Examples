use chrono::NaiveDateTime;
use diesel::{Identifiable, Insertable, Queryable, Selectable};
use uuid::Uuid;

use crate::schema::comments;

#[derive(Debug, Queryable, Identifiable, Selectable)]
pub struct Comment {
    pub id: i32,
    pub article_id: Uuid,
    pub user_id: Uuid,
    pub body: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = comments)]
pub struct NewComment {
    pub article_id: Uuid,
    pub user_id: Uuid,
    pub body: String,
}
