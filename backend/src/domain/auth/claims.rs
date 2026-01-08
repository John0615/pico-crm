use crate::domain::models::user::User;
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtClaims {
    pub sub: String,
    pub user_name: String,
    pub exp: i64,
}

impl JwtClaims {
    pub fn from_user(user: &User, expiry_hours: i64) -> Self {
        let expiration = Utc::now() + Duration::hours(expiry_hours);
        Self {
            sub: user.uuid.clone(),
            user_name: user.user_name.clone(),
            exp: expiration.timestamp(),
        }
    }

    pub fn is_valid(&self) -> bool {
        self.exp > Utc::now().timestamp()
    }
}
