use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct User {
    pub uuid: String,
    pub user_name: String,
    pub email: Option<String>,
    pub phone_number: Option<String>,
    pub is_admin: Option<bool>,
    pub status: String,
    pub avatar_url: Option<String>,
    pub last_login_at: Option<String>,
    pub email_verified_at: Option<String>,
    pub inserted_at: String,
    pub updated_at: String,
}
