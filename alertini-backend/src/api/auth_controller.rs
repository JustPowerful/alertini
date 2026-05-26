use argon2::{
    Argon2,
    password_hash::{ PasswordHash, PasswordVerifier, PasswordHasher, SaltString, rand_core::OsRng},
};

// use PasswordHash, PasswordVerifier, to verify the password later
use axum::{
    Json, Router, extract::State, http::StatusCode, routing::post
};

use chrono::{Duration, Utc};
use diesel::{ExpressionMethods, OptionalExtension, RunQueryDsl, SelectableHelper, query_dsl::methods::FilterDsl};
use jsonwebtoken::{EncodingKey, Header, encode};
use serde::Serialize;

use crate::{
    db::Pool, common::claims::Claims, models::user::{LoginUser, NewUser, User, UserResponse}, schema::users
};

use crate::responses::api_response::ApiResponse;

pub struct AuthController;



#[derive(Serialize)]
pub struct LoginResponse {
    token: String,
}

impl AuthController {
    pub fn app() -> Router<Pool> {
        Router::new()
        .route("/register", post(Self::register))
        .route("/login", post(Self::login))
    }

    pub async fn login(State(pool): State<Pool>, Json(body): Json<LoginUser>) -> (StatusCode, Json<ApiResponse<LoginResponse>>) {
        let mut conn = pool.get().expect("There was a problem getting the database pool.");
        let user: User = users::table.filter(users::email.eq(body.email)).first::<User>(&mut conn)
            .expect("User not found");

        let parsed_hash = PasswordHash::new(&user.password).expect("Error parsing password hash");
        let is_valid = Argon2::default().verify_password(body.password.as_bytes(), &parsed_hash).is_ok();

        if !is_valid {
            return (StatusCode::UNAUTHORIZED, Json(ApiResponse::error("Invalid email or password")))
        }
        
        
        let expiration = Utc::now()
            .checked_add_signed(Duration::hours(24))
            .unwrap()
            .timestamp() as usize;
        let claims = Claims {
            sub: user.id.to_string(),
            exp: expiration
        };
        let secret = std::env::var("JWT_SECRET_KEY").expect("There's not JWT_SECRET_KEY, please define it.");
        let token = encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_bytes())).expect("Error encoding the JWT token");
        
        return (StatusCode::OK, Json(ApiResponse::success("Successfully logged in", LoginResponse {token})))
    }

    pub async fn register(
        State(pool): State<Pool>,
        Json(body): Json<NewUser>,
    ) -> (StatusCode, Json<ApiResponse<UserResponse>>) {
        let pool_clone = pool.clone();
        let mut conn = pool_clone.get().expect("Failed to get connection");

        // We have to check if the email exists in the database
        let existing_user = users::table
            .filter(users::email.eq(&body.email))
            .first::<User>(&mut conn)
            .optional()
            .expect("There was a problem in register.");
        
        if existing_user.is_some() {
            return (StatusCode::CONFLICT, Json(ApiResponse::error("User already exists.")));
        }

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
        .expect("There was a problem inserting the user!");

        return (StatusCode::OK, Json(ApiResponse::success("Successfully registered your account.", UserResponse {
                id: user.id,
                firstname: user.firstname,
                lastname: user.lastname,
                email: user.email,
                created_at: user.created_at,
            })));
    }
}
