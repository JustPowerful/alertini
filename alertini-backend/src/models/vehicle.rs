use chrono::NaiveDateTime;
use diesel::{Selectable, associations::Associations, deserialize::Queryable, prelude::Insertable};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::schema::{vehicles};
use crate::models::user::User;

// Model Section

#[derive(Queryable, Selectable, Associations, Serialize)]
#[diesel(table_name = vehicles, belongs_to(User))]
pub struct Vehicle {
    pub id: Uuid,
    pub license_plate: String,
    pub car_desc: Option<String>,
    pub user_id: Uuid,
    pub created_at: Option<NaiveDateTime>
}

#[derive(Insertable)]
#[diesel(table_name = vehicles)]
pub struct NewVehicle {
    pub license_plate: String,
    pub car_desc: Option<String>,
    pub user_id: Uuid
}


// Json Payloads Section
#[derive(Deserialize)]
pub struct NewVehiclePayload {
    pub license_plate: String,
    pub car_desc: Option<String>
}

