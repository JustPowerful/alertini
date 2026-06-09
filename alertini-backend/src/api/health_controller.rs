use axum::{Json, Router, http::StatusCode, routing::get};


use crate::{
    api::{
        alert_controller::AlertController, auth_controller::AuthController,
        vehicle_controller::VehicleController,
    },
    db::{Pool, create_pool},
    responses::api_response::ApiResponse,
};
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
        Router::new().route("/", get(root_handler))
    }
}

#[utoipa::path(
        get,
        path = "/health",
        tag = "Health",
        responses(
            (status = 200, description = "Service health status for all subsystems")
        )
    )]
pub async fn root_handler() -> (StatusCode, Json<ApiResponse<HealthState>>) {
    
    let (db_service, pool) = create_pool().await;
    let auth_service = AuthController::health(&pool).await;
    let vehicle_service = VehicleController::health(&pool).await;
    let alert_service = AlertController::health(&pool).await;

    (
        StatusCode::OK,
        Json(ApiResponse::success(
            "Fetched service health data.",
            HealthState {
                alert_service,
                auth_service,
                vehicle_service,
                db_service,
            },
        )),
    )
}

