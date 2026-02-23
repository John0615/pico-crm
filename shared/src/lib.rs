use serde::{Deserialize, Serialize};

pub mod contact;
pub mod user;
pub mod file;
pub mod merchant;
pub mod auth;
pub mod system_config;
pub mod analytics;
pub mod admin;
pub mod service_request;
pub mod order;
pub mod schedule;
pub mod merchant_dashboard;

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct ListResult<T> {
    pub total: u64,
    pub items: Vec<T>,
}
