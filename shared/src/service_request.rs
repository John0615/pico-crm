use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct ServiceRequest {
    pub uuid: String,
    pub contact_uuid: String,
    pub creator_uuid: String,
    #[serde(default)]
    pub contact_name: Option<String>,
    #[serde(default)]
    pub creator_name: Option<String>,
    pub service_content: String,
    pub appointment_start_at: Option<String>,
    pub appointment_end_at: Option<String>,
    pub status: String,
    pub source: String,
    pub notes: Option<String>,
    pub inserted_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CreateServiceRequest {
    pub contact_uuid: String,
    pub service_content: String,
    pub appointment_start_at: Option<String>,
    pub appointment_end_at: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UpdateServiceRequest {
    pub uuid: String,
    pub service_content: String,
    pub appointment_start_at: Option<String>,
    pub appointment_end_at: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UpdateServiceRequestStatus {
    pub status: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ServiceRequestQuery {
    pub page: u64,
    pub page_size: u64,
    pub status: Option<String>,
    pub contact_uuid: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
}
