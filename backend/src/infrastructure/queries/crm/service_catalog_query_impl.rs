use sea_orm::entity::prelude::*;
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder,
};
use shared::service_catalog::{ServiceCatalog as SharedServiceCatalog, ServiceCatalogQuery};

use crate::domain::crm::service_catalog::ServiceCatalogQuery as DomainServiceCatalogQuery;
use crate::infrastructure::entity::service_catalogs::{
    Column as ServiceCatalogColumn, Entity as ServiceCatalogEntity,
};
use crate::infrastructure::mappers::crm::service_catalog_mapper::ServiceCatalogMapper;
use crate::infrastructure::tenant::{parse_merchant_uuid, with_shared_txn};

pub struct SeaOrmServiceCatalogQuery {
    db: DatabaseConnection,
    merchant_id: String,
}

impl SeaOrmServiceCatalogQuery {
    pub fn new(db: DatabaseConnection, merchant_id: String) -> Self {
        Self { db, merchant_id }
    }
}

impl DomainServiceCatalogQuery for SeaOrmServiceCatalogQuery {
    type Result = SharedServiceCatalog;

    fn list_service_catalogs(
        &self,
        query: ServiceCatalogQuery,
    ) -> impl std::future::Future<Output = Result<(Vec<Self::Result>, u64), String>> + Send {
        let db = self.db.clone();
        let merchant_id = self.merchant_id.clone();
        async move {
            with_shared_txn(&db, |txn| {
                let merchant_id = merchant_id.clone();
                Box::pin(async move {
                    let merchant_uuid = parse_merchant_uuid(&merchant_id)?;
                    let mut select = build_service_catalog_select(merchant_uuid, &query);

                    select = select
                        .order_by_asc(ServiceCatalogColumn::SortOrder)
                        .order_by_asc(ServiceCatalogColumn::InsertedAt);

                    let paginator = select.paginate(txn, query.page_size);

                    let items = paginator
                        .fetch_page(query.page - 1)
                        .await
                        .map_err(|e| format!("query service catalogs error: {}", e))?;

                    let total = paginator
                        .num_items()
                        .await
                        .map_err(|e| format!("count service catalogs error: {}", e))?;

                    Ok((
                        items
                            .into_iter()
                            .map(ServiceCatalogMapper::to_view)
                            .collect(),
                        total,
                    ))
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
        let merchant_id = self.merchant_id.clone();
        async move {
            with_shared_txn(&db, |txn| {
                let merchant_id = merchant_id.clone();
                Box::pin(async move {
                    let merchant_uuid = parse_merchant_uuid(&merchant_id)?;
                    let uuid = Uuid::parse_str(&uuid)
                        .map_err(|e| format!("invalid service catalog uuid: {}", e))?;
                    let item = ServiceCatalogEntity::find()
                        .filter(ServiceCatalogColumn::MerchantId.eq(merchant_uuid))
                        .filter(ServiceCatalogColumn::Uuid.eq(uuid))
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

fn build_service_catalog_select(
    merchant_uuid: Uuid,
    query: &ServiceCatalogQuery,
) -> sea_orm::Select<ServiceCatalogEntity> {
    let mut select =
        ServiceCatalogEntity::find().filter(ServiceCatalogColumn::MerchantId.eq(merchant_uuid));
    if query.active_only.unwrap_or(false) {
        select = select.filter(ServiceCatalogColumn::IsActive.eq(true));
    }
    select
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::{DbBackend, QueryTrait};

    #[test]
    fn generated_sql_contains_merchant_scope_for_service_catalog_list() {
        let merchant_uuid =
            Uuid::parse_str("11111111-1111-1111-1111-111111111111").expect("valid uuid");
        let query = ServiceCatalogQuery {
            page: 1,
            page_size: 20,
            active_only: Some(true),
        };

        let sql = build_service_catalog_select(merchant_uuid, &query)
            .build(DbBackend::Postgres)
            .to_string();

        assert!(
            sql.contains(
                r#""service_catalogs"."merchant_id" = '11111111-1111-1111-1111-111111111111'"#
            ),
            "sql: {sql}"
        );
        assert!(
            sql.contains(r#""service_catalogs"."is_active" = TRUE"#),
            "sql: {sql}"
        );
    }
}
