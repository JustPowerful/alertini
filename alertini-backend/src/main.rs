use axum::{Router, routing::get};
use tower_http::cors::CorsLayer;

// imported modules
mod api;
mod common;
mod db;
mod middleware;
mod models;
mod responses;
mod schema;

// controllers
use api::auth_controller::AuthController;
use api::vehicle_controller::VehicleController;
use db::create_pool;

use crate::api::alert_controller::AlertController;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok(); // initialize the dotenv variables from .env file
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = create_pool(database_url);

    // Inside every route
    // You can create a protected route, you can use the following
    // route_layer(middleware::from_fn(auth_middleware))
    let app = Router::new()
        .route("/", get(|| async { "Hello World " }))
        .nest("/vehicle", VehicleController::app())
        .nest("/auth", AuthController::app())
        .nest("/alert", AlertController::app())
        .with_state(pool)
        .layer(CorsLayer::permissive());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
