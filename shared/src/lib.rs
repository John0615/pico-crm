use serde::{Deserialize, Serialize};

pub mod contact;
pub mod user;
pub mod file;
pub mod merchant;
pub mod auth;
pub mod system_config;

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct ListResult<T> {
    pub total: u64,
    pub items: Vec<T>,
}
