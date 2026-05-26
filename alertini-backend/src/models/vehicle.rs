use chrono::NaiveDateTime;
use diesel::{Selectable, associations::Associations, deserialize::Queryable, prelude::Insertable};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::schema::{vehicles};
use crate::models::user::User;

#[derive(Queryable, Selectable, Associations, Serialize)]
#[belongs_to(User)]
#[diesel(table_name = vehicles)]
pub struct Vehicle {
    pub id: Uuid,
    pub license_plate: String,
    pub car_desc: Option<String>,
    pub user_id: Uuid,
    pub created_at: Option<NaiveDateTime>
}

#[derive(Insertable, Deserialize)]
#[diesel(table_name = vehicles)]
pub struct NewVehicle {
    pub license_plate: String,
    pub car_desc: Option<String>
}

