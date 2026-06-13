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

use common::addons::utoipa_auth_addon::SecurityAddon;

// controllers
use api::auth_controller::AuthController;
use api::vehicle_controller::VehicleController;
use db::create_pool;

use crate::api::alert_controller::AlertController;
use crate::api::health_controller::HealthController;

use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use utoipauto::utoipauto;



// SwaggerUI Documentation for the API
#[utoipauto(paths = "./src/api")]
#[derive(OpenApi)]
#[openapi(
    components(
        schemas(crate::models::vehicle::Vehicle,
            crate::models::vehicle::NewVehiclePayload,
            crate::models::user::User,
            crate::models::user::UserResponse,
            crate::models::user::LoginUser,
            crate::models::user::NewUser,
            crate::models::alert::Alert,
            crate::models::alert::NewAlert,
            crate::models::alert::NewAlertPayload,
            crate::models::alert::GetVehicleAlertPayload,
        )
    ),
    modifiers(&SecurityAddon)
)]
pub struct ApiDoc;         


use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok(); // initialize the dotenv variables from .env file
    // Get current present diesel SQL migrations 
    let (_, pool) = create_pool().await;
    
    let environment = std::env::var("ENV").unwrap_or("production".to_string());

    if environment == "production" {
        println!("ENV: PRODUCTION ENVIRONMENT -- MIGRATING DATABASE");
        pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

        let mut conn = pool.get().expect("db connection failed");
    
        // Run database SQL migrations
        conn.run_pending_migrations(MIGRATIONS)
            .expect("Failed to run migrations");
    }

    // Inside every route
    // You can create a protected route, you can use the following
    // route_layer(middleware::from_fn(auth_middleware))
    let app = Router::new()
        .route("/", get(|| async { "Hello World " }))
        .nest("/vehicle", VehicleController::app())
        .nest("/auth", AuthController::app())
        .nest("/alert", AlertController::app())
        .nest("/health", HealthController::app())
        .with_state(pool)
        .layer(CorsLayer::permissive())
        .merge(SwaggerUi::new("/docs").url("/api-doc/openapi.json", ApiDoc::openapi()));
    let port = std::env::var("PORT").unwrap_or("3000".to_string());
    let addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
