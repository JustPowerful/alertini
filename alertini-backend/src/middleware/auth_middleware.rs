use axum::{
    extract::Request,
    http::{StatusCode, header::AUTHORIZATION},
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode};

use crate::common::claims::Claims;

pub async fn auth_middleware(mut req: Request, next: Next) -> Result<Response, StatusCode> {
    let auth_header = req
        .headers()
        .get(AUTHORIZATION)
        .and_then(|header| header.to_str().ok());
    let auth_header = match auth_header {
        Some(header) if header.starts_with("Bearer ") => header,
        _ => return Err(StatusCode::UNAUTHORIZED),
    };

    let token = auth_header.trim_start_matches("Bearer ").trim();
    let secret =
        std::env::var("JWT_SECRET_KEY").expect("There's no JWT_SECRET_KET in env variables.");
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::new(Algorithm::HS256),
    )
    .map_err(|_| StatusCode::UNAUTHORIZED)
    .expect("Error decoding the token");

    // Store claims inside request extensions
    req.extensions_mut().insert(token_data.claims);

    Ok(next.run(req).await)
}
