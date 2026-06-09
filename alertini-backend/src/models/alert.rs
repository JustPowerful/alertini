use diesel::prelude::Insertable;
use diesel::{Associations, Queryable, Selectable};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use utoipa::ToSchema;

use crate::models::user::User;
use crate::models::vehicle::Vehicle;
use crate::schema::alerts;

// Model Section
#[derive(Debug, Clone, Queryable, Selectable, Associations, Serialize, ToSchema)]
#[diesel(table_name = alerts, belongs_to(User, foreign_key = reporter_id), belongs_to(Vehicle, foreign_key = car_id))]
pub struct Alert {
    pub id: Uuid,
    pub car_id: Uuid,
    pub note: String,
    pub reporter_id: Uuid,
}

#[derive(Insertable, ToSchema)]
#[diesel(table_name = alerts)]
pub struct NewAlert {
    pub car_id: Uuid,
    pub note: String,
    pub reporter_id: Uuid,
}

// Json payloads
#[allow(dead_code)]
#[derive(Deserialize, ToSchema)]
pub struct NewAlertPayload {
    pub license_plate: String,
    pub note: String,
}


#[derive(Deserialize, ToSchema)]
pub struct GetVehicleAlertPayload {
    pub license_plate: String,
}