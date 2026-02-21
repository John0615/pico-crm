use crate::application::utils::parse_utc_time_to_string;
use crate::domain::models::service_request::ServiceRequest as DomainServiceRequest;
use shared::service_request::ServiceRequest as SharedServiceRequest;

impl From<DomainServiceRequest> for SharedServiceRequest {
    fn from(request: DomainServiceRequest) -> Self {
        Self {
            uuid: request.uuid,
            contact_uuid: request.contact_uuid,
            service_content: request.service_content,
            appointment_start_at: request
                .appointment_start_at
                .map(parse_utc_time_to_string),
            appointment_end_at: request
                .appointment_end_at
                .map(parse_utc_time_to_string),
            status: request.status.as_str().to_string(),
            source: request.source.as_str().to_string(),
            notes: request.notes,
            inserted_at: parse_utc_time_to_string(request.inserted_at),
            updated_at: parse_utc_time_to_string(request.updated_at),
        }
    }
}
