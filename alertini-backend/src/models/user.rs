use diesel::prelude::*;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

use chrono::NaiveDateTime;

use crate::schema::users;
#[derive(Queryable, Selectable, Serialize)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(table_name = users)]
pub struct User {
    pub id: Uuid,
    pub firstname: String,
    pub lastname: String,
    pub email: String,
    pub password: String,
    pub created_at: Option<NaiveDateTime>,
}

#[derive(Insertable, Deserialize)]
#[diesel(table_name = users)]
pub struct NewUser {
    pub firstname: String,
    pub lastname: String,
    pub email: String,
    pub password: String,
}

// Response type user
#[derive(Serialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub firstname: String,
    pub lastname: String,
    pub email: String,
    pub created_at: Option<NaiveDateTime>,
}