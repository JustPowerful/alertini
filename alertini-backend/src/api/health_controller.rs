use axum::{Json, Router, http::StatusCode, routing::get};

use crate::{db::Pool, responses::api_response::ApiResponse};
use serde::Serialize;


pub struct HealthController;

#[derive(Serialize)]
pub struct HealthState {
    alert_service: bool,
    auth_service: bool,
    vehicle_service: bool,
}

impl HealthController {
    pub fn app() -> Router<Pool> {
        Router::new()
            .route("/", get(Self::root))
    }

    pub async fn root() -> (StatusCode, Json<ApiResponse<HealthState>>){
        (StatusCode::OK, Json(ApiResponse::success("Fetched service health data.", HealthState {
            alert_service: true,
            auth_service: true,
            vehicle_service: true
        })))
    }
}