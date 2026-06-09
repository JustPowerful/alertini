use axum::{Json, Router, http::StatusCode, routing::get};
use diesel::{r2d2::{ConnectionManager}, PgConnection, RunQueryDsl};


use crate::{db::Pool, responses::api_response::ApiResponse};
use serde::Serialize;


pub struct HealthController;

#[derive(Serialize)]
pub struct HealthState {
    alert_service: bool,
    auth_service: bool,
    vehicle_service: bool,
    db_service: bool,
}

impl HealthController {
    pub fn app() -> Router<Pool> {
        Router::new()
            .route("/", get(Self::root))
    }

    pub async fn root() -> (StatusCode, Json<ApiResponse<HealthState>>){
        // Verify the health the database connection and SQL request, do something like "SELECT 1"
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let pool = Pool::new(ConnectionManager::<PgConnection>::new(database_url)).expect("Failed to create database pool");
        let mut conn = pool.get().expect("Failed to get database connection from pool");

        let db_service = diesel::select(diesel::dsl::sql::<diesel::sql_types::Integer>("1"))
            .first::<i32>(&mut conn)
            .is_ok();

        (StatusCode::OK, Json(ApiResponse::success("Fetched service health data.", HealthState {
            alert_service: true,
            auth_service: true,
            vehicle_service: true,
            db_service,
        })))
    }
}