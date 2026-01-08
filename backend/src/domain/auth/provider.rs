use crate::domain::models::user::User;
use std::fmt::Debug;

#[derive(Debug, Clone)]
pub struct AuthCredential(pub String);

#[async_trait::async_trait]
pub trait AuthProvider: Debug + Clone {
    async fn authenticate(&self, username: &str, password: &str) -> Result<AuthCredential, String>;

    async fn get_current_user(&self, credential: &AuthCredential) -> Result<Option<User>, String>;

    async fn invalidate_credential(&self, credential: &AuthCredential) -> Result<(), String>;
}
