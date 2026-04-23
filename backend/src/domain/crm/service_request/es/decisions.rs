use chrono::{DateTime, Utc};
use disintegrate::Decision;

use super::events::{ServiceRequestEventEnvelope, seed_created_event};
use super::state::ServiceRequestState;
use crate::domain::crm::service_request::model::{
    ServiceRequest, ServiceRequestStatus, UpdateServiceRequest,
};

pub struct CreateServiceRequestDecision {
    merchant_id: String,
    request: ServiceRequest,
}

impl CreateServiceRequestDecision {
    pub fn new(merchant_id: impl Into<String>, request: ServiceRequest) -> Self {
        Self {
            merchant_id: merchant_id.into(),
            request,
        }
    }
}

impl Decision for CreateServiceRequestDecision {
    type Event = ServiceRequestEventEnvelope;
    type StateQuery = ServiceRequestState;
    type Error = String;

    fn state_query(&self) -> Self::StateQuery {
        ServiceRequestState::new(&self.merchant_id, &self.request.uuid)
    }

    fn process(&self, state: &Self::StateQuery) -> Result<Vec<Self::Event>, Self::Error> {
        if state.exists {
            return Err(format!(
                "service request {} already exists",
                self.request.uuid
            ));
        }

        self.request.verify()?;

        Ok(vec![seed_created_event(&self.merchant_id, &self.request)])
    }
}

pub struct UpdateServiceRequestDecision {
    merchant_id: String,
    update: UpdateServiceRequest,
    updated_at: DateTime<Utc>,
}

impl UpdateServiceRequestDecision {
    pub fn new(
        merchant_id: impl Into<String>,
        update: UpdateServiceRequest,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            merchant_id: merchant_id.into(),
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
        ServiceRequestState::new(&self.merchant_id, &self.update.uuid)
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
                merchant_id: self.merchant_id.clone(),
                request_uuid: self.update.uuid.clone(),
                service_catalog_uuid: self.update.service_catalog_uuid.clone(),
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
    merchant_id: String,
    request_uuid: String,
    next_status: ServiceRequestStatus,
    updated_at: DateTime<Utc>,
}

impl UpdateServiceRequestStatusDecision {
    pub fn new(
        merchant_id: impl Into<String>,
        request_uuid: impl Into<String>,
        next_status: ServiceRequestStatus,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            merchant_id: merchant_id.into(),
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
        ServiceRequestState::new(&self.merchant_id, &self.request_uuid)
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
                merchant_id: self.merchant_id.clone(),
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
            service_catalog_uuid: Some("11111111-1111-1111-1111-111111111111".to_string()),
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
                "11111111-1111-1111-1111-111111111111",
                request.clone(),
            ))
            .then([seed_created_event(
                "11111111-1111-1111-1111-111111111111",
                &request,
            )]);
    }

    #[test]
    fn it_updates_service_request_details() {
        let update = UpdateServiceRequest {
            uuid: "request-1".to_string(),
            service_catalog_uuid: Some("22222222-2222-2222-2222-222222222222".to_string()),
            service_content: "Updated visit".to_string(),
            appointment_start_at: Some(ts(2)),
            appointment_end_at: Some(ts(2) + chrono::Duration::hours(2)),
            notes: Some("updated".to_string()),
        };

        TestHarness::given([seed_created_event(
            "11111111-1111-1111-1111-111111111111",
            &sample_request(),
        )])
        .when(UpdateServiceRequestDecision::new(
            "11111111-1111-1111-1111-111111111111",
            update,
            ts(2),
        ))
        .then([ServiceRequestEventEnvelope::ServiceRequestDetailsUpdated {
            merchant_id: "11111111-1111-1111-1111-111111111111".to_string(),
            request_uuid: "request-1".to_string(),
            service_catalog_uuid: Some("22222222-2222-2222-2222-222222222222".to_string()),
            service_content: "Updated visit".to_string(),
            appointment_start_at: Some(ts(2)),
            appointment_end_at: Some(ts(2) + chrono::Duration::hours(2)),
            notes: Some("updated".to_string()),
            updated_at: ts(2),
        }]);
    }

    #[test]
    fn it_rejects_invalid_status_transition() {
        TestHarness::given([seed_created_event(
            "11111111-1111-1111-1111-111111111111",
            &sample_request(),
        )])
        .when(UpdateServiceRequestStatusDecision::new(
            "11111111-1111-1111-1111-111111111111",
            "request-1",
            ServiceRequestStatus::Converted,
            ts(3),
        ))
        .then_err("invalid status transition: new -> converted".to_string());
    }
}
