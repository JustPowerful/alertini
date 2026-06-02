use diesel::prelude::Insertable;
use uuid::Uuid;
use diesel::{Queryable, Selectable, Associations};
use serde::{Deserialize,Serialize};

use crate::models::user::User;
use crate::models::vehicle::Vehicle;
use crate::schema::alerts;

// Model Section
#[derive(Debug, Clone, Queryable, Selectable, Associations, Serialize)]
#[diesel(table_name = alerts, belongs_to(User, foreign_key = reporter_id), belongs_to(Vehicle, foreign_key = car_id))]
pub struct Alert {
    pub id: Uuid,
    pub car_id: Uuid,
    pub note: String,
    pub reporter_id: Uuid,
}

#[derive(Insertable)]
#[diesel(table_name = alerts)]
pub struct NewAlert {
    pub car_id: Uuid,
    pub note: String,
    pub reporter_id: Uuid,
}

// Json payloads
#[allow(dead_code)]
#[derive(Deserialize)]
pub struct NewAlertPayload {
    pub license_plate: String,
    pub note: String,
}