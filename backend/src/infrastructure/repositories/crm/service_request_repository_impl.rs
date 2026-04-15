use async_trait::async_trait;
use chrono::Utc;
use disintegrate::NoSnapshot;
use sea_orm::DatabaseConnection;

use crate::domain::crm::service_request::{
    CreateServiceRequestDecision, ServiceRequest, ServiceRequestRepository, ServiceRequestStatus,
    UpdateServiceRequest, UpdateServiceRequestDecision, UpdateServiceRequestStatusDecision,
};
use crate::infrastructure::event_store::service_request::event_store;

pub struct SeaOrmServiceRequestRepository {
    _db: DatabaseConnection,
    schema_name: String,
}

impl SeaOrmServiceRequestRepository {
    pub fn new(db: DatabaseConnection, schema_name: String) -> Self {
        Self {
            _db: db,
            schema_name,
        }
    }
}

#[async_trait]
impl ServiceRequestRepository for SeaOrmServiceRequestRepository {
    fn create_service_request(
        &self,
        request: ServiceRequest,
    ) -> impl std::future::Future<Output = Result<String, String>> + Send {
        let schema_name = self.schema_name.clone();
        async move {
            let request_uuid = request.uuid.clone();
            let event_store = event_store().await?;
            let decision_maker = disintegrate_postgres::decision_maker(event_store, NoSnapshot);
            decision_maker
                .make(CreateServiceRequestDecision::new(
                    schema_name.clone(),
                    request.clone(),
                ))
                .await
                .map_err(|e| format!("create service request decision error: {}", e))?;
            Ok(request_uuid)
        }
    }

    fn update_service_request(
        &self,
        request: UpdateServiceRequest,
    ) -> impl std::future::Future<Output = Result<(), String>> + Send {
        let schema_name = self.schema_name.clone();
        async move {
            let updated_at = Utc::now();
            let event_store = event_store().await?;
            let decision_maker = disintegrate_postgres::decision_maker(event_store, NoSnapshot);
            decision_maker
                .make(UpdateServiceRequestDecision::new(
                    schema_name,
                    request,
                    updated_at,
                ))
                .await
                .map_err(|e| format!("update service request decision error: {}", e))?;
            Ok(())
        }
    }

    fn update_service_request_status(
        &self,
        uuid: String,
        status: String,
    ) -> impl std::future::Future<Output = Result<(), String>> + Send {
        let schema_name = self.schema_name.clone();
        async move {
            let next_status = ServiceRequestStatus::parse(&status)?;
            let updated_at = Utc::now();
            let event_store = event_store().await?;
            let decision_maker = disintegrate_postgres::decision_maker(event_store, NoSnapshot);
            decision_maker
                .make(UpdateServiceRequestStatusDecision::new(
                    schema_name,
                    uuid,
                    next_status,
                    updated_at,
                ))
                .await
                .map_err(|e| format!("update service request status decision error: {}", e))?;
            Ok(())
        }
    }
}
