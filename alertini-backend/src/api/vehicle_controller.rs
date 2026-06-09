use std::str::FromStr;

use axum::{
    Json, Router,
    extract::{Extension, Path, State},
    http::StatusCode,
    middleware,
    routing::{delete, get, patch, post},
};
use diesel::{
    BoolExpressionMethods, ExpressionMethods, OptionalExtension, RunQueryDsl, SelectableHelper,
    query_dsl::methods::FilterDsl,
};
use uuid::Uuid;

use crate::{
    common::claims::Claims,
    db::Pool,
    middleware::auth_middleware::auth_middleware,
    models::vehicle::{NewVehicle, NewVehiclePayload, Vehicle},
    responses::api_response::ApiResponse,
    schema::vehicles,
};

pub struct VehicleController;

impl VehicleController {
    pub fn app() -> Router<Pool> {
        let protected_routes = Router::new()
            .route("/create", post(create_vehicle))
            .route("/delete/{vehicle_id}", delete(del_vehicle))
            .route("/update/{vehicle_id}", patch(update_vehicle))
            .route("/getall", get(get_all_vehicles))
            .route_layer(middleware::from_fn(auth_middleware));

        Router::new().merge(protected_routes)
    }

    pub async fn health(pool: &Pool) -> bool {
        pool.get().is_ok()
    }
}

#[utoipa::path(
    get,
    path = "/vehicle/getall",
    tag = "Vehicles",
    security(("api_jwt_token" = [])),
    responses(
        (status = 200, description = "List of all vehicles for the authenticated user"),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn get_all_vehicles(
    State(pool): State<Pool>,
    Extension(claims): Extension<Claims>,
) -> (StatusCode, Json<ApiResponse<Vec<Vehicle>>>) {
    let pool_clone = pool.clone();
    let mut conn = pool_clone.get().expect("Failed to get connection");
    let user_uuid = Uuid::from_str(&claims.sub).expect("Failed to parse user UUID");
    let vehicles: Vec<Vehicle> = vehicles::table
        .filter(vehicles::user_id.eq(user_uuid))
        .load::<Vehicle>(&mut conn)
        .expect("Failed to load vehicles");
    (
        StatusCode::OK,
        Json(ApiResponse::success(
            "Vehicles retrieved successfully.",
            vehicles,
        )),
    )
}

#[utoipa::path(
    post,
    path = "/vehicle/create",
    tag = "Vehicles",
    security(("api_jwt_token" = [])),
    request_body = NewVehiclePayload,
    responses(
        (status = 200, description = "Vehicle created successfully"),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn create_vehicle(
    State(pool): State<Pool>,
    Extension(claims): Extension<Claims>,
    Json(body): Json<NewVehiclePayload>,
) -> (StatusCode, Json<ApiResponse<Vehicle>>) {
    let pool_clone = pool.clone();
    let mut conn = pool_clone.get().expect("Failed to get connection");
    let new_vehicle = NewVehicle {
        license_plate: body.license_plate,
        car_desc: body.car_desc,
        user_id: Uuid::from_str(&claims.sub).expect("Failed to parse UUID"),
    };

    let vehicle: Vehicle = diesel::insert_into(vehicles::table)
        .values(&new_vehicle)
        .returning(Vehicle::as_returning())
        .get_result(&mut conn)
        .expect("There was a problem inserting the user!");

    (
        StatusCode::OK,
        Json(ApiResponse::success(
            "Successfully registered your account.",
            vehicle,
        )),
    )
}

#[utoipa::path(
    patch,
    path = "/vehicle/update/{vehicle_id}",
    tag = "Vehicles",
    security(("api_jwt_token" = [])),
    params(
        ("vehicle_id" = String, Path, description = "UUID of the vehicle to update")
    ),
    request_body = NewVehiclePayload,
    responses(
        (status = 200, description = "Vehicle updated successfully"),
        (status = 403, description = "Forbidden — vehicle does not belong to user"),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn update_vehicle(
    State(pool): State<Pool>,
    Path(vehicle_id): Path<String>,
    Extension(claims): Extension<Claims>,
    Json(body): Json<NewVehiclePayload>,
) -> (StatusCode, Json<ApiResponse<Vehicle>>) {
    let pool_clone = pool.clone();
    let mut conn = pool_clone.get().expect("Failed to get connection.");

    let vehicle_uuid = Uuid::from_str(&vehicle_id).expect("Failed to parse vehicle UUID");
    let user_uuid = Uuid::from_str(&claims.sub).expect("Failed to parse user UUID");

    let user_has_vehicle = vehicles::table
        .filter(
            vehicles::id
                .eq(vehicle_uuid)
                .and(vehicles::user_id.eq(user_uuid)),
        )
        .first::<Vehicle>(&mut conn)
        .optional()
        .expect("There was a problem in deleting vehicle.");

    if !user_has_vehicle.is_some() {
        return (
            StatusCode::FORBIDDEN,
            Json(ApiResponse::error("You cannot update this vehicle")),
        );
    }

    let vehicle: Vehicle = diesel::update(vehicles::table.filter(vehicles::id.eq(vehicle_uuid)))
        .set((
            vehicles::license_plate.eq(&body.license_plate),
            vehicles::car_desc.eq(&body.car_desc),
        ))
        .returning(Vehicle::as_returning())
        .get_result(&mut conn)
        .expect("Failed to update vehicle");

    (
        StatusCode::OK,
        Json(ApiResponse::success(
            "Vehicle updated successfully.",
            vehicle,
        )),
    )
}

#[utoipa::path(
    delete,
    path = "/vehicle/delete/{vehicle_id}",
    tag = "Vehicles",
    security(("api_jwt_token" = [])),
    params(
        ("vehicle_id" = String, Path, description = "UUID of the vehicle to delete")
    ),
    responses(
        (status = 200, description = "Vehicle deleted successfully"),
        (status = 403, description = "Forbidden — vehicle does not belong to user"),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn del_vehicle(
    State(pool): State<Pool>,
    Path(vehicle_id): Path<String>,
    Extension(claims): Extension<Claims>,
) -> (StatusCode, Json<ApiResponse<Vehicle>>) {
    let pool_clone = pool.clone();
    let mut conn = pool_clone.get().expect("Failed to get connection");

    let vehicle_uuid = Uuid::from_str(&vehicle_id).expect("Failed to parse vehicle UUID");
    let user_uuid = Uuid::from_str(&claims.sub).expect("Failed to parse user UUID");

    let user_has_vehicle = vehicles::table
        .filter(
            vehicles::id
                .eq(vehicle_uuid)
                .and(vehicles::user_id.eq(user_uuid)),
        )
        .first::<Vehicle>(&mut conn)
        .optional()
        .expect("There was a problem in deleting vehicle.");

    if !user_has_vehicle.is_some() {
        return (
            StatusCode::FORBIDDEN,
            Json(ApiResponse::error("You cannot remove this vehicle")),
        );
    }

    let vehicle: Vehicle = diesel::delete(vehicles::table.filter(vehicles::id.eq(vehicle_uuid)))
        .returning(Vehicle::as_returning())
        .get_result(&mut conn)
        .expect("Failed to delete vehicle");
    (
        StatusCode::OK,
        Json(ApiResponse::success(
            "Vehicle deleted successfully.",
            vehicle,
        )),
    )
}
