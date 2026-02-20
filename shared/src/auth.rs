use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LoginResponse {
    pub role: String,
    pub redirect_to: String,
}
