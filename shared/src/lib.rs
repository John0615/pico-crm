use serde::{Deserialize, Serialize};

pub mod contact;

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct ListResult<T> {
    pub total: u64,
    pub contacts: Vec<T>,
}
