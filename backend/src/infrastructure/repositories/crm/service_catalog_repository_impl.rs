use async_trait::async_trait;
use sea_orm::entity::prelude::*;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, QueryFilter};

use crate::domain::crm::service_catalog::{
    ServiceCatalog, ServiceCatalogRepository, UpdateServiceCatalog,
};
use crate::infrastructure::entity::service_catalogs::{
    Column as ServiceCatalogColumn, Entity as ServiceCatalogEntity,
};
use crate::infrastructure::mappers::crm::service_catalog_mapper::ServiceCatalogMapper;
use crate::infrastructure::tenant::with_tenant_txn;

pub struct SeaOrmServiceCatalogRepository {
    db: DatabaseConnection,
    schema_name: String,
}

impl SeaOrmServiceCatalogRepository {
    pub fn new(db: DatabaseConnection, schema_name: String) -> Self {
        Self { db, schema_name }
    }
}

#[async_trait]
impl ServiceCatalogRepository for SeaOrmServiceCatalogRepository {
    fn create_service_catalog(
        &self,
        catalog: ServiceCatalog,
    ) -> impl std::future::Future<Output = Result<ServiceCatalog, String>> + Send {
        let db = self.db.clone();
        let schema_name = self.schema_name.clone();
        async move {
            with_tenant_txn(&db, &schema_name, |txn| {
                Box::pin(async move {
                    let active = ServiceCatalogMapper::to_active_entity(catalog);
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
        let schema_name = self.schema_name.clone();
        async move {
            with_tenant_txn(&db, &schema_name, |txn| {
                Box::pin(async move {
                    let uuid = Uuid::parse_str(&catalog.uuid)
                        .map_err(|e| format!("invalid service catalog uuid: {}", e))?;
                    let original = ServiceCatalogEntity::find()
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
}
