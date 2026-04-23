use async_trait::async_trait;
use sea_orm::entity::prelude::*;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, QueryFilter};

use crate::domain::crm::service_catalog::{
    ServiceCatalog, ServiceCatalogRepository, UpdateServiceCatalog,
};
use crate::infrastructure::entity::service_catalogs::{
    Column as ServiceCatalogColumn, Entity as ServiceCatalogEntity,
};
use crate::infrastructure::entity::service_requests::Column as ServiceRequestColumn;
use crate::infrastructure::entity::service_requests::Entity as ServiceRequestEntity;
use crate::infrastructure::mappers::crm::service_catalog_mapper::ServiceCatalogMapper;
use crate::infrastructure::tenant::{parse_merchant_uuid, with_shared_txn};

pub struct SeaOrmServiceCatalogRepository {
    db: DatabaseConnection,
    merchant_id: String,
}

impl SeaOrmServiceCatalogRepository {
    pub fn new(db: DatabaseConnection, merchant_id: String) -> Self {
        Self { db, merchant_id }
    }
}

#[async_trait]
impl ServiceCatalogRepository for SeaOrmServiceCatalogRepository {
    fn create_service_catalog(
        &self,
        catalog: ServiceCatalog,
    ) -> impl std::future::Future<Output = Result<ServiceCatalog, String>> + Send {
        let db = self.db.clone();
        let merchant_id = self.merchant_id.clone();
        async move {
            with_shared_txn(&db, |txn| {
                let merchant_id = merchant_id.clone();
                Box::pin(async move {
                    let merchant_uuid = parse_merchant_uuid(&merchant_id)?;
                    let mut active = ServiceCatalogMapper::to_active_entity(catalog);
                    active.merchant_id = sea_orm::ActiveValue::Set(Some(merchant_uuid));
                    let created = active
                        .insert(txn)
                        .await
                        .map_err(|e| format!("create service catalog error: {}", e))?;
                    Ok(ServiceCatalogMapper::to_domain(created))
                })
            })
            .await
        }
    }

    fn update_service_catalog(
        &self,
        catalog: UpdateServiceCatalog,
    ) -> impl std::future::Future<Output = Result<ServiceCatalog, String>> + Send {
        let db = self.db.clone();
        let merchant_id = self.merchant_id.clone();
        async move {
            with_shared_txn(&db, |txn| {
                let merchant_id = merchant_id.clone();
                Box::pin(async move {
                    let merchant_uuid = parse_merchant_uuid(&merchant_id)?;
                    let uuid = Uuid::parse_str(&catalog.uuid)
                        .map_err(|e| format!("invalid service catalog uuid: {}", e))?;
                    let original = ServiceCatalogEntity::find()
                        .filter(ServiceCatalogColumn::MerchantId.eq(merchant_uuid))
                        .filter(ServiceCatalogColumn::Uuid.eq(uuid))
                        .one(txn)
                        .await
                        .map_err(|e| format!("query service catalog error: {}", e))?
                        .ok_or_else(|| "service catalog not found".to_string())?;

                    let active = ServiceCatalogMapper::to_update_active_entity(catalog, original);
                    let updated = active
                        .update(txn)
                        .await
                        .map_err(|e| format!("update service catalog error: {}", e))?;
                    Ok(ServiceCatalogMapper::to_domain(updated))
                })
            })
            .await
        }
    }

    fn delete_service_catalog(
        &self,
        uuid: String,
    ) -> impl std::future::Future<Output = Result<(), String>> + Send {
        let db = self.db.clone();
        let merchant_id = self.merchant_id.clone();
        async move {
            with_shared_txn(&db, |txn| {
                let merchant_id = merchant_id.clone();
                Box::pin(async move {
                    let merchant_uuid = parse_merchant_uuid(&merchant_id)?;
                    let uuid = Uuid::parse_str(&uuid)
                        .map_err(|e| format!("invalid service catalog uuid: {}", e))?;
                    let entity = ServiceCatalogEntity::find()
                        .filter(ServiceCatalogColumn::MerchantId.eq(merchant_uuid))
                        .filter(ServiceCatalogColumn::Uuid.eq(uuid))
                        .one(txn)
                        .await
                        .map_err(|e| format!("query service catalog error: {}", e))?
                        .ok_or_else(|| "service catalog not found".to_string())?;

                    let active: crate::infrastructure::entity::service_catalogs::ActiveModel =
                        entity.into();
                    active
                        .delete(txn)
                        .await
                        .map_err(|e| format!("delete service catalog error: {}", e))?;
                    Ok(())
                })
            })
            .await
        }
    }

    fn is_service_catalog_in_use(
        &self,
        uuid: String,
    ) -> impl std::future::Future<Output = Result<bool, String>> + Send {
        let db = self.db.clone();
        let merchant_id = self.merchant_id.clone();
        async move {
            with_shared_txn(&db, |txn| {
                let merchant_id = merchant_id.clone();
                Box::pin(async move {
                    let merchant_uuid = parse_merchant_uuid(&merchant_id)?;
                    let uuid = Uuid::parse_str(&uuid)
                        .map_err(|e| format!("invalid service catalog uuid: {}", e))?;
                    let count = ServiceRequestEntity::find()
                        .filter(ServiceRequestColumn::MerchantId.eq(merchant_uuid))
                        .filter(ServiceRequestColumn::ServiceCatalogUuid.eq(uuid))
                        .count(txn)
                        .await
                        .map_err(|e| format!("query service requests error: {}", e))?;
                    Ok(count > 0)
                })
            })
            .await
        }
    }
}
