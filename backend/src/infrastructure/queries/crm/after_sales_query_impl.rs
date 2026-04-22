use std::collections::HashMap;

use async_trait::async_trait;
use sea_orm::entity::prelude::*;
use sea_orm::{
    ColumnTrait, DatabaseConnection, DatabaseTransaction, EntityTrait, QueryFilter, QueryOrder,
};
use shared::after_sales::AfterSalesCase as SharedAfterSalesCase;
use shared::after_sales::AfterSalesCaseRecord as SharedAfterSalesCaseRecord;

use crate::domain::crm::after_sales::AfterSalesCaseQuery;
use crate::infrastructure::entity::after_sales_case_records::{
    Column as AfterSalesRecordColumn, Entity as AfterSalesRecordEntity,
};
use crate::infrastructure::entity::after_sales_cases::{
    Column as AfterSalesColumn, Entity as AfterSalesEntity,
};
use crate::infrastructure::entity::users::{Column as UserColumn, Entity as UserEntity};
use crate::infrastructure::mappers::crm::after_sales_mapper::AfterSalesCaseMapper;
use crate::infrastructure::mappers::crm::after_sales_record_mapper::AfterSalesCaseRecordMapper;
use crate::infrastructure::tenant::with_tenant_txn;

pub struct SeaOrmAfterSalesCaseQuery {
    db: DatabaseConnection,
    schema_name: String,
}

impl SeaOrmAfterSalesCaseQuery {
    pub fn new(db: DatabaseConnection, schema_name: String) -> Self {
        Self { db, schema_name }
    }
}

#[async_trait]
impl AfterSalesCaseQuery for SeaOrmAfterSalesCaseQuery {
    type Result = SharedAfterSalesCase;
    type RecordResult = SharedAfterSalesCaseRecord;

    fn list_cases(
        &self,
        order_uuid: String,
    ) -> impl std::future::Future<Output = Result<Vec<Self::Result>, String>> + Send {
        let db = self.db.clone();
        let schema_name = self.schema_name.clone();
        async move {
            with_tenant_txn(&db, &schema_name, |txn| {
                Box::pin(async move {
                    let order_uuid = Uuid::parse_str(&order_uuid)
                        .map_err(|e| format!("invalid order uuid: {}", e))?;

                    let items = AfterSalesEntity::find()
                        .filter(AfterSalesColumn::OrderUuid.eq(order_uuid))
                        .order_by_desc(AfterSalesColumn::InsertedAt)
                        .all(txn)
                        .await
                        .map_err(|e| format!("query after sales cases error: {}", e))?;

                    let operator_names = load_operator_names(
                        txn,
                        items.iter().filter_map(|item| item.operator_uuid).collect(),
                    )
                    .await?;

                    Ok(items
                        .into_iter()
                        .map(|item| {
                            let operator_name = item
                                .operator_uuid
                                .and_then(|uuid| operator_names.get(&uuid).cloned());
                            AfterSalesCaseMapper::to_view(item, operator_name)
                        })
                        .collect())
                })
            })
            .await
        }
    }

    fn list_case_records(
        &self,
        case_uuid: String,
    ) -> impl std::future::Future<Output = Result<Vec<Self::RecordResult>, String>> + Send {
        let db = self.db.clone();
        let schema_name = self.schema_name.clone();
        async move {
            with_tenant_txn(&db, &schema_name, |txn| {
                Box::pin(async move {
                    let case_uuid = Uuid::parse_str(&case_uuid)
                        .map_err(|e| format!("invalid case uuid: {}", e))?;

                    let items = AfterSalesRecordEntity::find()
                        .filter(AfterSalesRecordColumn::CaseUuid.eq(case_uuid))
                        .order_by_desc(AfterSalesRecordColumn::InsertedAt)
                        .all(txn)
                        .await
                        .map_err(|e| format!("query after sales records error: {}", e))?;

                    let operator_names = load_operator_names(
                        txn,
                        items.iter().filter_map(|item| item.operator_uuid).collect(),
                    )
                    .await?;

                    Ok(items
                        .into_iter()
                        .map(|item| {
                            let operator_name = item
                                .operator_uuid
                                .and_then(|uuid| operator_names.get(&uuid).cloned());
                            AfterSalesCaseRecordMapper::to_view(item, operator_name)
                        })
                        .collect())
                })
            })
            .await
        }
    }
}

async fn load_operator_names(
    txn: &DatabaseTransaction,
    operator_ids: Vec<Uuid>,
) -> Result<HashMap<Uuid, String>, String> {
    let operator_ids = operator_ids.into_iter().collect::<Vec<_>>();
    if operator_ids.is_empty() {
        return Ok(HashMap::new());
    }

    let users = UserEntity::find()
        .filter(UserColumn::Uuid.is_in(operator_ids))
        .all(txn)
        .await
        .map_err(|e| format!("query after sales operators error: {}", e))?;

    Ok(users
        .into_iter()
        .map(|user| (user.uuid, user.user_name))
        .collect())
}
