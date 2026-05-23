use argon2::{
    Argon2,
    password_hash::{ PasswordHasher, SaltString, rand_core::OsRng},
};
// use PasswordHash, PasswordVerifier, to verify the password later
use axum::{
    Json, Router,
    extract::State,
    routing::{post},
};
use diesel::{RunQueryDsl, SelectableHelper};
use serde::Serialize;

use crate::{
    db::Pool,
    models::user::{NewUser, User, UserResponse},
    schema::users,
};
pub struct AuthController;

#[derive(Serialize)]
pub struct JsonResponse<T> {
    success: bool,
    message: String,
    data: T,
}

impl AuthController {
    pub fn app() -> Router<Pool> {
        Router::new()
            .route("/register", post(Self::register))
    }

    pub async fn register(
        State(pool): State<Pool>,
        Json(body): Json<NewUser>,
    ) -> Json<JsonResponse<UserResponse>> {
        // Get a connection from the pool
        let mut conn = pool.get().expect("Failed to get connection");


        // Hash the password using Argon2
        /*
         * We pass OsRng as a mutable reference because it's a stateful random number generator that needs to maintain internal state.
         * It's like making a function that takes a mutable reference to a random number generator, so it can generate random numbers and update its internal state accordingly.
         */
        
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2.hash_password(body.password.as_bytes(), &salt)
            .expect("Failed to hash password")
            .to_string();

        let user_data = NewUser {
            firstname: body.firstname,
            lastname: body.lastname,
            email: body.email,
            password: password_hash    
        };

        let user: User = diesel::insert_into(users::table)
            .values(user_data)
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
                created_at: user.created_at,
            },
        })
    }
}
