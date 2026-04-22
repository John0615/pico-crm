use sea_orm::entity::prelude::*;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder};
use shared::service_catalog::{ServiceCatalog as SharedServiceCatalog, ServiceCatalogQuery};

use crate::domain::crm::service_catalog::ServiceCatalogQuery as DomainServiceCatalogQuery;
use crate::infrastructure::entity::service_catalogs::{
    Column as ServiceCatalogColumn, Entity as ServiceCatalogEntity,
};
use crate::infrastructure::mappers::crm::service_catalog_mapper::ServiceCatalogMapper;
use crate::infrastructure::tenant::with_tenant_txn;

pub struct SeaOrmServiceCatalogQuery {
    db: DatabaseConnection,
    schema_name: String,
}

impl SeaOrmServiceCatalogQuery {
    pub fn new(db: DatabaseConnection, schema_name: String) -> Self {
        Self { db, schema_name }
    }
}

impl DomainServiceCatalogQuery for SeaOrmServiceCatalogQuery {
    type Result = SharedServiceCatalog;

    fn list_service_catalogs(
        &self,
        query: ServiceCatalogQuery,
    ) -> impl std::future::Future<Output = Result<Vec<Self::Result>, String>> + Send {
        let db = self.db.clone();
        let schema_name = self.schema_name.clone();
        async move {
            with_tenant_txn(&db, &schema_name, |txn| {
                Box::pin(async move {
                    let mut select = ServiceCatalogEntity::find();
                    if query.active_only.unwrap_or(false) {
                        select = select.filter(ServiceCatalogColumn::IsActive.eq(true));
                    }

                    let items = select
                        .order_by_asc(ServiceCatalogColumn::SortOrder)
                        .order_by_asc(ServiceCatalogColumn::InsertedAt)
                        .all(txn)
                        .await
                        .map_err(|e| format!("query service catalogs error: {}", e))?;

                    Ok(items
                        .into_iter()
                        .map(ServiceCatalogMapper::to_view)
                        .collect())
                })
            })
            .await
        }
    }

    fn get_service_catalog(
        &self,
        uuid: String,
    ) -> impl std::future::Future<Output = Result<Option<Self::Result>, String>> + Send {
        let db = self.db.clone();
        let schema_name = self.schema_name.clone();
        async move {
            with_tenant_txn(&db, &schema_name, |txn| {
                Box::pin(async move {
                    let uuid = Uuid::parse_str(&uuid)
                        .map_err(|e| format!("invalid service catalog uuid: {}", e))?;
                    let item = ServiceCatalogEntity::find_by_id(uuid)
                        .one(txn)
                        .await
                        .map_err(|e| format!("query service catalog error: {}", e))?;

                    Ok(item.map(ServiceCatalogMapper::to_view))
                })
            })
            .await
        }
    }
}
