use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use chrono::NaiveDateTime;

use crate::schema::users;
#[derive(Queryable, Selectable, Serialize, ToSchema)]
#[diesel(table_name = users)]
pub struct User {
    pub id: Uuid,
    pub firstname: String,
    pub lastname: String,
    pub email: String,
    pub password: String,
    pub created_at: Option<NaiveDateTime>,
}

#[derive(Insertable, Deserialize, ToSchema)]
#[diesel(table_name = users)]
pub struct NewUser {
    pub firstname: String,
    pub lastname: String,
    pub email: String,
    pub password: String,
}

#[derive(Queryable, Selectable, Serialize, Deserialize, ToSchema)]
#[diesel(table_name = users)]
pub struct LoginUser {
    pub email: String,
    pub password: String,
}

// Response type user
#[derive(Serialize, ToSchema)]
pub struct UserResponse {
    pub id: Uuid,
    pub firstname: String,
    pub lastname: String,
    pub email: String,
    pub created_at: Option<NaiveDateTime>,
}
