use axum::{Router, routing::{get}};

// imported modules
mod api;


// controllers 
use api::auth_controller::AuthController; 


#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(| | async { "Hello World "}))
        .nest("/auth", AuthController::app());
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}



