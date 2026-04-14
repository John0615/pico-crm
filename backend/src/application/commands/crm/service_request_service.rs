use crate::domain::crm::service_request::{
    ServiceRequest as DomainServiceRequest, ServiceRequestRepository, ServiceRequestStatus,
    UpdateServiceRequest,
};
use chrono::{DateTime, NaiveDate, NaiveDateTime, TimeZone, Utc};
use shared::service_request::{
    CreateServiceRequest, ServiceRequestCommandResult,
    UpdateServiceRequest as SharedUpdateServiceRequest, UpdateServiceRequestStatus,
};

pub struct ServiceRequestAppService<R: ServiceRequestRepository> {
    repo: R,
}

impl<R: ServiceRequestRepository> ServiceRequestAppService<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    pub async fn create_service_request(
        &self,
        request: CreateServiceRequest,
        creator_uuid: String,
    ) -> Result<ServiceRequestCommandResult, String> {
        let appointment_start_at = parse_datetime(request.appointment_start_at.as_deref());
        let appointment_end_at = parse_datetime(request.appointment_end_at.as_deref());
        let service_request = DomainServiceRequest::new(
            request.customer_uuid,
            creator_uuid,
            request.service_content,
            appointment_start_at,
            appointment_end_at,
            request.notes,
        )?;
        let uuid = self.repo.create_service_request(service_request).await?;
        Ok(ServiceRequestCommandResult { uuid })
    }

    pub async fn update_service_request(
        &self,
        request: SharedUpdateServiceRequest,
    ) -> Result<ServiceRequestCommandResult, String> {
        let uuid = request.uuid.clone();
        let appointment_start_at = parse_datetime(request.appointment_start_at.as_deref());
        let appointment_end_at = parse_datetime(request.appointment_end_at.as_deref());
        if request.service_content.trim().is_empty() {
            return Err("Service content is required".to_string());
        }
        if let (Some(start), Some(end)) = (appointment_start_at, appointment_end_at) {
            if end < start {
                return Err("Appointment end must be after start".to_string());
            }
        }
        let update = UpdateServiceRequest {
            uuid: request.uuid,
            service_content: request.service_content,
            appointment_start_at,
            appointment_end_at,
            notes: request.notes,
        };
        self.repo.update_service_request(update).await?;
        Ok(ServiceRequestCommandResult { uuid })
    }

    pub async fn update_service_request_status(
        &self,
        uuid: String,
        status: UpdateServiceRequestStatus,
    ) -> Result<ServiceRequestCommandResult, String> {
        ServiceRequestStatus::parse(&status.status)?;
        self.repo
            .update_service_request_status(uuid.clone(), status.status)
            .await?;
        Ok(ServiceRequestCommandResult { uuid })
    }
}

fn parse_datetime(value: Option<&str>) -> Option<DateTime<Utc>> {
    let Some(value) = value else { return None };
    if value.trim().is_empty() {
        return None;
    }
    if let Ok(dt) = DateTime::parse_from_rfc3339(value) {
        return Some(dt.with_timezone(&Utc));
    }
    if let Ok(dt) = NaiveDateTime::parse_from_str(value, "%Y-%m-%d %H:%M:%S") {
        return Some(Utc.from_utc_datetime(&dt));
    }
    if let Ok(date) = NaiveDate::parse_from_str(value, "%Y-%m-%d") {
        if let Some(dt) = date.and_hms_opt(0, 0, 0) {
            return Some(Utc.from_utc_datetime(&dt));
        }
    }
    None
}
