use axum::{Router, middleware, routing::get};


use crate::{db::Pool, middleware::auth_middleware::{auth_middleware}};

pub struct VehicleController;

impl VehicleController {
    pub fn app() -> Router<Pool> {
        let protected_routes = Router::new()
            .route("/", get(Self::vehicle_root))
            .route_layer(middleware::from_fn(auth_middleware));
        
        // Return the router
        Router::new()
            .merge(protected_routes)
            
    }

    pub async fn vehicle_root() -> &'static str {
        "Hello, World"
    }
}