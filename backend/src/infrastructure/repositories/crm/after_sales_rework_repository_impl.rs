use async_trait::async_trait;
use sea_orm::{ActiveModelTrait, DatabaseConnection};

use crate::domain::crm::after_sales_rework::{
    AfterSalesRework, AfterSalesReworkRepository, CreateAfterSalesRework,
};
use crate::infrastructure::mappers::crm::after_sales_rework_mapper::AfterSalesReworkMapper;
use crate::infrastructure::tenant::{parse_merchant_uuid, with_shared_txn};

pub struct SeaOrmAfterSalesReworkRepository {
    db: DatabaseConnection,
    merchant_id: String,
}

impl SeaOrmAfterSalesReworkRepository {
    pub fn new(db: DatabaseConnection, merchant_id: String) -> Self {
        Self { db, merchant_id }
    }
}

#[async_trait]
impl AfterSalesReworkRepository for SeaOrmAfterSalesReworkRepository {
    fn create_rework(
        &self,
        rework: CreateAfterSalesRework,
    ) -> impl std::future::Future<Output = Result<AfterSalesRework, String>> + Send {
        let db = self.db.clone();
        let merchant_id = self.merchant_id.clone();
        async move {
            with_shared_txn(&db, |txn| {
                let merchant_id = merchant_id.clone();
                Box::pin(async move {
                    let merchant_uuid = parse_merchant_uuid(&merchant_id)?;
                    let mut active = AfterSalesReworkMapper::to_active_entity(rework)?;
                    active.merchant_id = sea_orm::ActiveValue::Set(Some(merchant_uuid));
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
