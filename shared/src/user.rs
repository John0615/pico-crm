use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct User {
    pub uuid: String,
    pub user_name: String,
    pub email: Option<String>,
    pub phone_number: Option<String>,
    pub merchant_uuid: Option<String>,
    pub role: String,
    pub is_admin: Option<bool>,
    pub status: String,
    pub avatar_url: Option<String>,
    pub last_login_at: Option<String>,
    pub email_verified_at: Option<String>,
    pub inserted_at: String,
    pub updated_at: String,
}

/// 用户创建请求
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CreateUserRequest {
    pub user_name: String,
    pub password: String,
    pub email: Option<String>,
    pub phone_number: Option<String>,
    pub avatar_url: Option<String>,
    pub merchant_uuid: Option<String>,
    pub role: Option<String>,
}

/// 用户查询参数
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UserListQuery {
    pub page: u64,
    pub page_size: u64,
    pub name: Option<String>,
    pub role: Option<String>,
    pub status: Option<String>,
}

/// 分页结果
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PagedResult<T> {
    pub items: Vec<T>,
    pub total: u64,
}
