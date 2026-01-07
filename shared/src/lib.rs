use serde::{Deserialize, Serialize};

pub mod contact;
pub mod user;

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct ListResult<T> {
    pub total: u64,
    pub items: Vec<T>,
}
