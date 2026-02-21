use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct TenantMigrationRequest {
    pub status: Option<String>,
    pub merchant_uuid: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct TenantMigrationResponse {
    pub total: u64,
    pub migrated: u64,
    pub failed: u64,
    pub failures: Vec<TenantMigrationFailure>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TenantMigrationFailure {
    pub schema_name: String,
    pub error: String,
}
