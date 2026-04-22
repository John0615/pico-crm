use chrono::{DateTime, Utc};
use disintegrate::Event;
use serde::{Deserialize, Serialize};

use crate::domain::crm::service_request::model::ServiceRequest;

#[derive(Debug, Clone, PartialEq, Eq, Event, Serialize, Deserialize)]
#[stream(
    ServiceRequestEvent,
    [
        ServiceRequestCreated,
        ServiceRequestDetailsUpdated,
        ServiceRequestStatusChanged
    ]
)]
pub enum ServiceRequestEventEnvelope {
    ServiceRequestCreated {
        #[id]
        tenant_schema: String,
        #[id]
        request_uuid: String,
        customer_uuid: String,
        creator_uuid: String,
        service_catalog_uuid: Option<String>,
        service_content: String,
        appointment_start_at: Option<DateTime<Utc>>,
        appointment_end_at: Option<DateTime<Utc>>,
        status: String,
        source: String,
        notes: Option<String>,
        inserted_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    },
    ServiceRequestDetailsUpdated {
        #[id]
        tenant_schema: String,
        #[id]
        request_uuid: String,
        service_catalog_uuid: Option<String>,
        service_content: String,
        appointment_start_at: Option<DateTime<Utc>>,
        appointment_end_at: Option<DateTime<Utc>>,
        notes: Option<String>,
        updated_at: DateTime<Utc>,
    },
    ServiceRequestStatusChanged {
        #[id]
        tenant_schema: String,
        #[id]
        request_uuid: String,
        status: String,
        updated_at: DateTime<Utc>,
    },
}

pub fn seed_created_event(
    tenant_schema: &str,
    request: &ServiceRequest,
) -> ServiceRequestEventEnvelope {
    ServiceRequestEventEnvelope::ServiceRequestCreated {
        tenant_schema: tenant_schema.to_string(),
        request_uuid: request.uuid.clone(),
        customer_uuid: request.customer_uuid.clone(),
        creator_uuid: request.creator_uuid.clone(),
        service_catalog_uuid: request.service_catalog_uuid.clone(),
        service_content: request.service_content.clone(),
        appointment_start_at: request.appointment_start_at,
        appointment_end_at: request.appointment_end_at,
        status: request.status.as_str().to_string(),
        source: request.source.as_str().to_string(),
        notes: request.notes.clone(),
        inserted_at: request.inserted_at,
        updated_at: request.updated_at,
    }
}
