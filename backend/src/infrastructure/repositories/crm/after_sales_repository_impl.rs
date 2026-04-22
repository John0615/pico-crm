use async_trait::async_trait;
use sea_orm::entity::prelude::*;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, IntoActiveModel, QueryFilter};

use crate::domain::crm::after_sales::{
    AfterSalesCase, AfterSalesCaseRecord, AfterSalesCaseRepository, CreateAfterSalesCase,
    CreateAfterSalesCaseRecord, UpdateAfterSalesRefund,
};
use crate::infrastructure::entity::after_sales_cases::{
    Column as AfterSalesColumn, Entity as AfterSalesEntity,
};
use crate::infrastructure::mappers::crm::after_sales_mapper::AfterSalesCaseMapper;
use crate::infrastructure::mappers::crm::after_sales_record_mapper::AfterSalesCaseRecordMapper;
use crate::infrastructure::tenant::with_tenant_txn;

pub struct SeaOrmAfterSalesCaseRepository {
    db: DatabaseConnection,
    schema_name: String,
}

impl SeaOrmAfterSalesCaseRepository {
    pub fn new(db: DatabaseConnection, schema_name: String) -> Self {
        Self { db, schema_name }
    }
}

#[async_trait]
impl AfterSalesCaseRepository for SeaOrmAfterSalesCaseRepository {
    fn create_case(
        &self,
        case: CreateAfterSalesCase,
    ) -> impl std::future::Future<Output = Result<AfterSalesCase, String>> + Send {
        let db = self.db.clone();
        let schema_name = self.schema_name.clone();
        async move {
            with_tenant_txn(&db, &schema_name, |txn| {
                Box::pin(async move {
                    let active = AfterSalesCaseMapper::to_active_entity(case)?;
                    let created = active
                        .insert(txn)
                        .await
                        .map_err(|e| format!("create after sales case error: {}", e))?;

                    Ok(AfterSalesCaseMapper::to_domain(created, None))
                })
            })
            .await
        }
    }

    fn update_refund(
        &self,
        refund: UpdateAfterSalesRefund,
    ) -> impl std::future::Future<Output = Result<AfterSalesCase, String>> + Send {
        let db = self.db.clone();
        let schema_name = self.schema_name.clone();
        async move {
            with_tenant_txn(&db, &schema_name, |txn| {
                Box::pin(async move {
                    let case_uuid = Uuid::parse_str(&refund.case_uuid)
                        .map_err(|e| format!("invalid case_uuid: {}", e))?;
                    let case_model = AfterSalesEntity::find()
                        .filter(AfterSalesColumn::Uuid.eq(case_uuid))
                        .one(txn)
                        .await
                        .map_err(|e| format!("query after sales case error: {}", e))?
                        .ok_or_else(|| "after sales case not found".to_string())?;

                    let mut case_active = case_model.into_active_model();
                    case_active.refund_amount_cents =
                        sea_orm::ActiveValue::Set(refund.refund_amount_cents);
                    case_active.refund_reason =
                        sea_orm::ActiveValue::Set(refund.refund_reason.clone());
                    case_active.updated_at = sea_orm::ActiveValue::Set(chrono::Utc::now());
                    let updated = case_active
                        .update(txn)
                        .await
                        .map_err(|e| format!("update after sales refund error: {}", e))?;

                    Ok(AfterSalesCaseMapper::to_domain(updated, None))
                })
            })
            .await
        }
    }

    fn create_case_record(
        &self,
        record: CreateAfterSalesCaseRecord,
    ) -> impl std::future::Future<Output = Result<AfterSalesCaseRecord, String>> + Send {
        let db = self.db.clone();
        let schema_name = self.schema_name.clone();
        async move {
            with_tenant_txn(&db, &schema_name, |txn| {
                Box::pin(async move {
                    let case_uuid = Uuid::parse_str(&record.case_uuid)
                        .map_err(|e| format!("invalid case_uuid: {}", e))?;
                    let next_status = record
                        .status
                        .clone()
                        .unwrap_or_else(|| "processing".to_string());

                    let case_model = AfterSalesEntity::find()
                        .filter(AfterSalesColumn::Uuid.eq(case_uuid))
                        .one(txn)
                        .await
                        .map_err(|e| format!("query after sales case error: {}", e))?
                        .ok_or_else(|| "after sales case not found".to_string())?;

                    let mut case_active = case_model.into_active_model();
                    case_active.status = sea_orm::ActiveValue::Set(next_status.clone());
                    case_active.updated_at = sea_orm::ActiveValue::Set(chrono::Utc::now());
                    case_active
                        .update(txn)
                        .await
                        .map_err(|e| format!("update after sales case status error: {}", e))?;

                    let active = AfterSalesCaseRecordMapper::to_active_entity(record)?;
                    let created = active
                        .insert(txn)
                        .await
                        .map_err(|e| format!("create after sales record error: {}", e))?;

                    Ok(AfterSalesCaseRecordMapper::to_domain(created, None))
                })
            })
            .await
        }
    }
}
