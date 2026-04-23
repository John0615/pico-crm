use serde::{Deserialize, Serialize};

pub mod after_sales;
pub mod analytics;
pub mod auth;
pub mod contact;
pub mod file;
pub mod merchant;
pub mod merchant_dashboard;
pub mod order;
pub mod schedule;
pub mod service_catalog;
pub mod service_request;
pub mod system_config;
pub mod user;

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct ListResult<T> {
    pub total: u64,
    pub items: Vec<T>,
}
