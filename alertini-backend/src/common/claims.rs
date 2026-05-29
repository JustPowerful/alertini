use serde::{Deserialize, Serialize};

// Claims are used in JWT token auth

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}
