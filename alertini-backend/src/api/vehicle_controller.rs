use axum::{extract::Extension, Router, middleware, routing::get};


use crate::{common::claims::Claims, db::Pool, middleware::auth_middleware::auth_middleware};

pub struct VehicleController;

impl VehicleController {
    pub fn app() -> Router<Pool> {
        // Separate protected routes for validating auth
        let protected_routes = Router::new()
            .route("/", get(Self::vehicle_root))
            .route_layer(middleware::from_fn(auth_middleware));
        
        // Return the router
        Router::new()
            .merge(protected_routes)
            
    }


    // you can get the middleware requestion extension using the Extension extractor
    pub async fn vehicle_root(Extension(claims): Extension<Claims>) -> String {
       format!("User ID: {}", claims.sub)
    }
}