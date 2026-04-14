use crate::domain::identity::user::User;
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtClaims {
    pub sub: String,
    pub user_name: String,
    pub merchant_id: String,
    pub role: String,
    pub exp: i64,
}

impl JwtClaims {
    pub fn from_user(user: &User, merchant_id: String, role: String, expiry_hours: i64) -> Self {
        let expiration = Utc::now() + Duration::hours(expiry_hours);
        Self {
            sub: user.uuid.clone(),
            user_name: user.user_name.clone(),
            merchant_id,
            role,
            exp: expiration.timestamp(),
        }
    }

    pub fn is_valid(&self) -> bool {
        self.exp > Utc::now().timestamp()
    }
}
