use chrono::NaiveDateTime;
use diesel::{Selectable, deserialize::Queryable, prelude::Insertable};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::schema::{vehicles};

#[derive(Queryable, Selectable, Serialize)]
#[diesel(table_name = vehicles)]
pub struct Vehicle {
    pub id: Uuid,
    pub license_plate: String,
    pub car_desc: Option<String>,
    pub created_at: Option<NaiveDateTime>
}

#[derive(Insertable, Deserialize)]
#[diesel(table_name = vehicles)]
pub struct NewVehicle {
    pub license_plate: String,
    pub car_desc: Option<String>
}

