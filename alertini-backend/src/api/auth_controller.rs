use axum::{Router, routing::get};

pub struct AuthController;

impl AuthController {
    pub fn app() -> Router {
        Router::new()
            .route("/", get(Self::get_auth_root))
    }

    pub async fn get_auth_root() -> &'static str {
        return "Auth Controller Root";
    }
}

