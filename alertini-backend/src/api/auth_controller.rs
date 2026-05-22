use axum::{Json, Router, routing::{get, post}, extract::State};
use diesel::{RunQueryDsl, SelectableHelper};
use serde::Serialize;

use crate::{db::Pool, models::user::{NewUser, User, UserResponse}, schema::users};
pub struct AuthController;

#[derive(Serialize)]
pub struct JsonResponse<T> {
    success: bool,
    message: String,
    data: T
}

impl AuthController {
    pub fn app() -> Router<Pool> {
        Router::new()
            .route("/", get(Self::get_auth_root))
            .route("/register", post(Self::register))
    }

    pub async fn get_auth_root() -> &'static str {
        return "Auth Controller Root";
    }

    pub async fn register(
        State(pool): State<Pool>,
        Json(body): Json<NewUser>,
    ) -> Json<JsonResponse<UserResponse>> {
        // Get a connection from the pool
        let mut conn = pool.get().expect("Failed to get connection");
        
        let user: User = diesel::insert_into(users::table)
            .values(&body)
            .returning(User::as_returning())
            .get_result(&mut conn)
            .expect("Failed to insert user");
        
        Json(JsonResponse {
            success: true,
            message: String::from("Successfully registered!"),
            data: UserResponse {
                id: user.id,
                firstname: user.firstname,
                lastname: user.lastname,
                email: user.email,
                created_at: user.created_at
            }
        })
    }
}

