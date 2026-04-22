use async_trait::async_trait;
use sea_orm::{ActiveModelTrait, DatabaseConnection};

use crate::domain::crm::after_sales_rework::{
    AfterSalesRework, AfterSalesReworkRepository, CreateAfterSalesRework,
};
use crate::infrastructure::mappers::crm::after_sales_rework_mapper::AfterSalesReworkMapper;
use crate::infrastructure::tenant::with_tenant_txn;

pub struct SeaOrmAfterSalesReworkRepository {
    db: DatabaseConnection,
    schema_name: String,
}

impl SeaOrmAfterSalesReworkRepository {
    pub fn new(db: DatabaseConnection, schema_name: String) -> Self {
        Self { db, schema_name }
    }
}

#[async_trait]
impl AfterSalesReworkRepository for SeaOrmAfterSalesReworkRepository {
    fn create_rework(
        &self,
        rework: CreateAfterSalesRework,
    ) -> impl std::future::Future<Output = Result<AfterSalesRework, String>> + Send {
        let db = self.db.clone();
        let schema_name = self.schema_name.clone();
        async move {
            with_tenant_txn(&db, &schema_name, |txn| {
                Box::pin(async move {
                    let active = AfterSalesReworkMapper::to_active_entity(rework)?;
                    let created = active
                        .insert(txn)
                        .await
                        .map_err(|e| format!("create after sales rework error: {}", e))?;
                    Ok(AfterSalesReworkMapper::to_domain(created, None))
                })
            })
            .await
        }
    }
}
