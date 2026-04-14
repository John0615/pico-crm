use chrono::{DateTime, Utc};
use disintegrate::{StateMutate, StateQuery};
use serde::{Deserialize, Serialize};

use super::events::ServiceRequestEvent;
use crate::domain::crm::service_request::model::{
    ServiceRequest, ServiceRequestSource, ServiceRequestStatus,
};

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize, StateQuery)]
#[state_query(ServiceRequestEvent)]
pub struct ServiceRequestState {
    #[id]
    pub tenant_schema: String,
    #[id]
    pub request_uuid: String,
    pub exists: bool,
    pub customer_uuid: Option<String>,
    pub creator_uuid: Option<String>,
    pub service_content: String,
    pub appointment_start_at: Option<DateTime<Utc>>,
    pub appointment_end_at: Option<DateTime<Utc>>,
    pub status: Option<String>,
    pub source: Option<String>,
    pub notes: Option<String>,
    pub inserted_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl ServiceRequestState {
    pub fn new(tenant_schema: impl Into<String>, request_uuid: impl Into<String>) -> Self {
        Self {
            tenant_schema: tenant_schema.into(),
            request_uuid: request_uuid.into(),
            ..Default::default()
        }
    }

    pub fn to_domain(&self) -> Result<ServiceRequest, String> {
        if !self.exists {
            return Err(format!("service request {} not found", self.request_uuid));
        }

        let customer_uuid = self
            .customer_uuid
            .clone()
            .ok_or_else(|| "service request customer is missing".to_string())?;
        let creator_uuid = self
            .creator_uuid
            .clone()
            .ok_or_else(|| "service request creator is missing".to_string())?;
        let status = ServiceRequestStatus::parse(
            self.status
                .as_deref()
                .unwrap_or(ServiceRequestStatus::New.as_str()),
        )?;
        let source = ServiceRequestSource::parse(
            self.source
                .as_deref()
                .unwrap_or(ServiceRequestSource::SalesManual.as_str()),
        )?;
        let inserted_at = self
            .inserted_at
            .ok_or_else(|| "service request inserted_at is missing".to_string())?;
        let updated_at = self
            .updated_at
            .ok_or_else(|| "service request updated_at is missing".to_string())?;

        Ok(ServiceRequest {
            uuid: self.request_uuid.clone(),
            customer_uuid,
            creator_uuid,
            service_content: self.service_content.clone(),
            appointment_start_at: self.appointment_start_at,
            appointment_end_at: self.appointment_end_at,
            status,
            source,
            notes: self.notes.clone(),
            inserted_at,
            updated_at,
        })
    }
}

impl StateMutate for ServiceRequestState {
    fn mutate(&mut self, event: Self::Event) {
        match event {
            ServiceRequestEvent::ServiceRequestCreated {
                tenant_schema,
                request_uuid,
                customer_uuid,
                creator_uuid,
                service_content,
                appointment_start_at,
                appointment_end_at,
                status,
                source,
                notes,
                inserted_at,
                updated_at,
            } => {
                self.exists = true;
                self.tenant_schema = tenant_schema;
                self.request_uuid = request_uuid;
                self.customer_uuid = Some(customer_uuid);
                self.creator_uuid = Some(creator_uuid);
                self.service_content = service_content;
                self.appointment_start_at = appointment_start_at;
                self.appointment_end_at = appointment_end_at;
                self.status = Some(status);
                self.source = Some(source);
                self.notes = notes;
                self.inserted_at = Some(inserted_at);
                self.updated_at = Some(updated_at);
            }
            ServiceRequestEvent::ServiceRequestDetailsUpdated {
                service_content,
                appointment_start_at,
                appointment_end_at,
                notes,
                updated_at,
                ..
            } => {
                self.service_content = service_content;
                self.appointment_start_at = appointment_start_at;
                self.appointment_end_at = appointment_end_at;
                self.notes = notes;
                self.updated_at = Some(updated_at);
            }
            ServiceRequestEvent::ServiceRequestStatusChanged {
                status, updated_at, ..
            } => {
                self.status = Some(status);
                self.updated_at = Some(updated_at);
            }
        }
    }
}
