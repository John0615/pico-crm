use async_trait::async_trait;
use sea_orm::entity::prelude::*;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait};

use crate::domain::models::service_request::{ServiceRequest, ServiceRequestStatus, UpdateServiceRequest};
use crate::domain::repositories::service_request::ServiceRequestRepository;
use crate::infrastructure::entity::service_requests::Entity;
use crate::infrastructure::mappers::service_request_mapper::ServiceRequestMapper;
use crate::infrastructure::tenant::with_tenant_txn;

pub struct SeaOrmServiceRequestRepository {
    db: DatabaseConnection,
    schema_name: String,
}

impl SeaOrmServiceRequestRepository {
    pub fn new(db: DatabaseConnection, schema_name: String) -> Self {
        Self { db, schema_name }
    }
}

#[async_trait]
impl ServiceRequestRepository for SeaOrmServiceRequestRepository {
    fn create_service_request(
        &self,
        request: ServiceRequest,
    ) -> impl std::future::Future<Output = Result<ServiceRequest, String>> + Send {
        let db = self.db.clone();
        let schema_name = self.schema_name.clone();
        async move {
            with_tenant_txn(&db, &schema_name, |txn| {
                Box::pin(async move {
                    let entity = ServiceRequestMapper::to_active_entity(request);
                    let inserted = entity
                        .insert(txn)
                        .await
                        .map_err(|e| format!("create service request error: {}", e))?;
                    Ok(ServiceRequestMapper::to_domain(inserted))
                })
            })
            .await
        }
    }

    fn update_service_request(
        &self,
        request: UpdateServiceRequest,
    ) -> impl std::future::Future<Output = Result<ServiceRequest, String>> + Send {
        let db = self.db.clone();
        let schema_name = self.schema_name.clone();
        async move {
            with_tenant_txn(&db, &schema_name, |txn| {
                Box::pin(async move {
                    let uuid = Uuid::parse_str(&request.uuid)
                        .map_err(|e| format!("invalid request uuid: {}", e))?;
                    let original = Entity::find_by_id(uuid)
                        .one(txn)
                        .await
                        .map_err(|e| format!("query service request error: {}", e))?
                        .ok_or_else(|| format!("service request {} not found", request.uuid))?;

                    let active = ServiceRequestMapper::to_update_active_entity(request, &original);
                    let updated = active
                        .update(txn)
                        .await
                        .map_err(|e| format!("update service request error: {}", e))?;
                    Ok(ServiceRequestMapper::to_domain(updated))
                })
            })
            .await
        }
    }

    fn update_service_request_status(
        &self,
        uuid: String,
        status: String,
    ) -> impl std::future::Future<Output = Result<ServiceRequest, String>> + Send {
        let db = self.db.clone();
        let schema_name = self.schema_name.clone();
        async move {
            let status = ServiceRequestStatus::parse(&status)?;
            with_tenant_txn(&db, &schema_name, |txn| {
                let status = status;
                let uuid = uuid.clone();
                Box::pin(async move {
                    let request_uuid = Uuid::parse_str(&uuid)
                        .map_err(|e| format!("invalid request uuid: {}", e))?;
                    let original = Entity::find_by_id(request_uuid)
                        .one(txn)
                        .await
                        .map_err(|e| format!("query service request error: {}", e))?
                        .ok_or_else(|| format!("service request {} not found", uuid))?;

                    let current_status = ServiceRequestStatus::parse(&original.status)
                        .unwrap_or(ServiceRequestStatus::New);
                    if !ServiceRequestStatus::can_transition(current_status, status) {
                        return Err(format!(
                            "invalid status transition: {} -> {}",
                            current_status.as_str(),
                            status.as_str()
                        ));
                    }

                    let active = ServiceRequestMapper::to_status_active_entity(original, status);
                    let updated = active
                        .update(txn)
                        .await
                        .map_err(|e| format!("update service request status error: {}", e))?;
                    Ok(ServiceRequestMapper::to_domain(updated))
                })
            })
            .await
        }
    }
}
