use crate::domain::models::service_request::{
    ServiceRequest, ServiceRequestSource, ServiceRequestStatus, UpdateServiceRequest,
};
use crate::infrastructure::entity::service_requests::{ActiveModel, Model};
use crate::infrastructure::utils::parse_date_time_to_string;
use chrono::Utc;
use sea_orm::ActiveValue::Set;
use sea_orm::entity::prelude::*;
use sea_orm::IntoActiveModel;
use shared::service_request::ServiceRequest as SharedServiceRequest;

pub struct ServiceRequestMapper;

impl ServiceRequestMapper {
    pub fn to_view(entity: Model) -> SharedServiceRequest {
        SharedServiceRequest {
            uuid: entity.uuid.to_string(),
            contact_uuid: entity.contact_uuid.to_string(),
            creator_uuid: entity.creator_uuid.to_string(),
            contact_name: None,
            creator_name: None,
            service_content: entity.service_content,
            appointment_start_at: entity
                .appointment_start_at
                .map(parse_date_time_to_string),
            appointment_end_at: entity.appointment_end_at.map(parse_date_time_to_string),
            status: entity.status,
            source: entity.source,
            notes: entity.notes,
            inserted_at: parse_date_time_to_string(entity.inserted_at),
            updated_at: parse_date_time_to_string(entity.updated_at),
        }
    }

    pub fn to_domain(entity: Model) -> ServiceRequest {
        let status = ServiceRequestStatus::parse(&entity.status)
            .unwrap_or(ServiceRequestStatus::New);
        let source = ServiceRequestSource::parse(&entity.source)
            .unwrap_or(ServiceRequestSource::SalesManual);
        ServiceRequest {
            uuid: entity.uuid.to_string(),
            contact_uuid: entity.contact_uuid.to_string(),
            creator_uuid: entity.creator_uuid.to_string(),
            service_content: entity.service_content,
            appointment_start_at: entity.appointment_start_at,
            appointment_end_at: entity.appointment_end_at,
            status,
            source,
            notes: entity.notes,
            inserted_at: entity.inserted_at,
            updated_at: entity.updated_at,
        }
    }

    pub fn to_active_entity(request: ServiceRequest) -> ActiveModel {
        ActiveModel {
            uuid: Set(Uuid::parse_str(&request.uuid).expect("Invalid UUID")),
            contact_uuid: Set(Uuid::parse_str(&request.contact_uuid).expect("Invalid contact UUID")),
            creator_uuid: Set(Uuid::parse_str(&request.creator_uuid).expect("Invalid creator UUID")),
            service_content: Set(request.service_content),
            appointment_start_at: Set(request.appointment_start_at),
            appointment_end_at: Set(request.appointment_end_at),
            status: Set(request.status.as_str().to_string()),
            source: Set(request.source.as_str().to_string()),
            notes: Set(request.notes),
            inserted_at: Set(request.inserted_at),
            updated_at: Set(request.updated_at),
        }
    }

    pub fn to_update_active_entity(update: UpdateServiceRequest, original: &Model) -> ActiveModel {
        let mut active = original.clone().into_active_model();
        active.service_content = Set(update.service_content);
        active.appointment_start_at = Set(update.appointment_start_at);
        active.appointment_end_at = Set(update.appointment_end_at);
        active.notes = Set(update.notes);
        active.updated_at = Set(Utc::now());
        active
    }

    pub fn to_status_active_entity(original: Model, status: ServiceRequestStatus) -> ActiveModel {
        let mut active = original.into_active_model();
        active.status = Set(status.as_str().to_string());
        active.updated_at = Set(Utc::now());
        active
    }
}
