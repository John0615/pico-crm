use chrono::{DateTime, Utc};
use disintegrate::Decision;

use super::events::{ServiceRequestEventEnvelope, seed_created_event};
use super::state::ServiceRequestState;
use crate::domain::crm::service_request::model::{
    ServiceRequest, ServiceRequestStatus, UpdateServiceRequest,
};

pub struct CreateServiceRequestDecision {
    tenant_schema: String,
    request: ServiceRequest,
}

impl CreateServiceRequestDecision {
    pub fn new(tenant_schema: impl Into<String>, request: ServiceRequest) -> Self {
        Self {
            tenant_schema: tenant_schema.into(),
            request,
        }
    }
}

impl Decision for CreateServiceRequestDecision {
    type Event = ServiceRequestEventEnvelope;
    type StateQuery = ServiceRequestState;
    type Error = String;

    fn state_query(&self) -> Self::StateQuery {
        ServiceRequestState::new(&self.tenant_schema, &self.request.uuid)
    }

    fn process(&self, state: &Self::StateQuery) -> Result<Vec<Self::Event>, Self::Error> {
        if state.exists {
            return Err(format!(
                "service request {} already exists",
                self.request.uuid
            ));
        }

        self.request.verify()?;

        Ok(vec![seed_created_event(&self.tenant_schema, &self.request)])
    }
}

pub struct UpdateServiceRequestDecision {
    tenant_schema: String,
    update: UpdateServiceRequest,
    updated_at: DateTime<Utc>,
}

impl UpdateServiceRequestDecision {
    pub fn new(
        tenant_schema: impl Into<String>,
        update: UpdateServiceRequest,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            tenant_schema: tenant_schema.into(),
            update,
            updated_at,
        }
    }
}

impl Decision for UpdateServiceRequestDecision {
    type Event = ServiceRequestEventEnvelope;
    type StateQuery = ServiceRequestState;
    type Error = String;

    fn state_query(&self) -> Self::StateQuery {
        ServiceRequestState::new(&self.tenant_schema, &self.update.uuid)
    }

    fn process(&self, state: &Self::StateQuery) -> Result<Vec<Self::Event>, Self::Error> {
        if !state.exists {
            return Err(format!("service request {} not found", self.update.uuid));
        }

        if self.update.service_content.trim().is_empty() {
            return Err("Service content is required".to_string());
        }
        if let (Some(start), Some(end)) = (
            self.update.appointment_start_at,
            self.update.appointment_end_at,
        ) {
            if end < start {
                return Err("Appointment end must be after start".to_string());
            }
        }

        Ok(vec![
            ServiceRequestEventEnvelope::ServiceRequestDetailsUpdated {
                tenant_schema: self.tenant_schema.clone(),
                request_uuid: self.update.uuid.clone(),
                service_content: self.update.service_content.clone(),
                appointment_start_at: self.update.appointment_start_at,
                appointment_end_at: self.update.appointment_end_at,
                notes: self.update.notes.clone(),
                updated_at: self.updated_at,
            },
        ])
    }
}

pub struct UpdateServiceRequestStatusDecision {
    tenant_schema: String,
    request_uuid: String,
    next_status: ServiceRequestStatus,
    updated_at: DateTime<Utc>,
}

impl UpdateServiceRequestStatusDecision {
    pub fn new(
        tenant_schema: impl Into<String>,
        request_uuid: impl Into<String>,
        next_status: ServiceRequestStatus,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            tenant_schema: tenant_schema.into(),
            request_uuid: request_uuid.into(),
            next_status,
            updated_at,
        }
    }
}

impl Decision for UpdateServiceRequestStatusDecision {
    type Event = ServiceRequestEventEnvelope;
    type StateQuery = ServiceRequestState;
    type Error = String;

    fn state_query(&self) -> Self::StateQuery {
        ServiceRequestState::new(&self.tenant_schema, &self.request_uuid)
    }

    fn process(&self, state: &Self::StateQuery) -> Result<Vec<Self::Event>, Self::Error> {
        if !state.exists {
            return Err(format!("service request {} not found", self.request_uuid));
        }

        let current_status = ServiceRequestStatus::parse(
            state
                .status
                .as_deref()
                .unwrap_or(ServiceRequestStatus::New.as_str()),
        )?;
        if !ServiceRequestStatus::can_transition(current_status, self.next_status) {
            return Err(format!(
                "invalid status transition: {} -> {}",
                current_status.as_str(),
                self.next_status.as_str()
            ));
        }

        Ok(vec![
            ServiceRequestEventEnvelope::ServiceRequestStatusChanged {
                tenant_schema: self.tenant_schema.clone(),
                request_uuid: self.request_uuid.clone(),
                status: self.next_status.as_str().to_string(),
                updated_at: self.updated_at,
            },
        ])
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;
    use disintegrate::TestHarness;

    use super::*;
    use crate::domain::crm::service_request::model::ServiceRequestSource;

    fn ts(day: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 3, day, 10, 0, 0)
            .single()
            .expect("valid timestamp")
    }

    fn sample_request() -> ServiceRequest {
        ServiceRequest {
            uuid: "request-1".to_string(),
            customer_uuid: "customer-1".to_string(),
            creator_uuid: "user-1".to_string(),
            service_content: "Initial visit".to_string(),
            appointment_start_at: Some(ts(1)),
            appointment_end_at: Some(ts(1) + chrono::Duration::hours(1)),
            status: ServiceRequestStatus::New,
            source: ServiceRequestSource::SalesManual,
            notes: Some("first note".to_string()),
            inserted_at: ts(1),
            updated_at: ts(1),
        }
    }

    #[test]
    fn it_creates_a_service_request() {
        let request = sample_request();

        TestHarness::given([])
            .when(CreateServiceRequestDecision::new(
                "tenant_a",
                request.clone(),
            ))
            .then([seed_created_event("tenant_a", &request)]);
    }

    #[test]
    fn it_updates_service_request_details() {
        let update = UpdateServiceRequest {
            uuid: "request-1".to_string(),
            service_content: "Updated visit".to_string(),
            appointment_start_at: Some(ts(2)),
            appointment_end_at: Some(ts(2) + chrono::Duration::hours(2)),
            notes: Some("updated".to_string()),
        };

        TestHarness::given([seed_created_event("tenant_a", &sample_request())])
            .when(UpdateServiceRequestDecision::new("tenant_a", update, ts(2)))
            .then([ServiceRequestEventEnvelope::ServiceRequestDetailsUpdated {
                tenant_schema: "tenant_a".to_string(),
                request_uuid: "request-1".to_string(),
                service_content: "Updated visit".to_string(),
                appointment_start_at: Some(ts(2)),
                appointment_end_at: Some(ts(2) + chrono::Duration::hours(2)),
                notes: Some("updated".to_string()),
                updated_at: ts(2),
            }]);
    }

    #[test]
    fn it_rejects_invalid_status_transition() {
        TestHarness::given([seed_created_event("tenant_a", &sample_request())])
            .when(UpdateServiceRequestStatusDecision::new(
                "tenant_a",
                "request-1",
                ServiceRequestStatus::Converted,
                ts(3),
            ))
            .then_err("invalid status transition: new -> converted".to_string());
    }
}
