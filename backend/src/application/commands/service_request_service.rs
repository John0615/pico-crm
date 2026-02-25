use chrono::{DateTime, NaiveDate, NaiveDateTime, TimeZone, Utc};
use crate::domain::models::service_request::{
    ServiceRequest as DomainServiceRequest, ServiceRequestStatus, UpdateServiceRequest,
};
use crate::domain::repositories::service_request::ServiceRequestRepository;
use shared::service_request::{
    CreateServiceRequest, ServiceRequest as SharedServiceRequest,
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
    ) -> Result<SharedServiceRequest, String> {
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
        let created = self.repo.create_service_request(service_request).await?;
        Ok(created.into())
    }

    pub async fn update_service_request(
        &self,
        request: SharedUpdateServiceRequest,
    ) -> Result<SharedServiceRequest, String> {
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
        let updated = self.repo.update_service_request(update).await?;
        Ok(updated.into())
    }

    pub async fn update_service_request_status(
        &self,
        uuid: String,
        status: UpdateServiceRequestStatus,
    ) -> Result<SharedServiceRequest, String> {
        ServiceRequestStatus::parse(&status.status)?;
        let updated = self
            .repo
            .update_service_request_status(uuid, status.status)
            .await?;
        Ok(updated.into())
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
