use std::collections::HashMap;

use async_trait::async_trait;
use sea_orm::entity::prelude::*;
use sea_orm::{
    ColumnTrait, DatabaseConnection, DatabaseTransaction, EntityTrait, QueryFilter, QueryOrder,
};
use shared::after_sales::AfterSalesRework as SharedAfterSalesRework;

use crate::domain::crm::after_sales_rework::AfterSalesReworkQuery;
use crate::infrastructure::entity::after_sales_reworks::{
    Column as AfterSalesReworkColumn, Entity as AfterSalesReworkEntity,
};
use crate::infrastructure::entity::users::{Column as UserColumn, Entity as UserEntity};
use crate::infrastructure::mappers::crm::after_sales_rework_mapper::AfterSalesReworkMapper;
use crate::infrastructure::tenant::with_tenant_txn;

pub struct SeaOrmAfterSalesReworkQuery {
    db: DatabaseConnection,
    schema_name: String,
}

impl SeaOrmAfterSalesReworkQuery {
    pub fn new(db: DatabaseConnection, schema_name: String) -> Self {
        Self { db, schema_name }
    }
}

#[async_trait]
impl AfterSalesReworkQuery for SeaOrmAfterSalesReworkQuery {
    type Result = SharedAfterSalesRework;

    fn list_reworks(
        &self,
        case_uuid: String,
    ) -> impl std::future::Future<Output = Result<Vec<Self::Result>, String>> + Send {
        let db = self.db.clone();
        let schema_name = self.schema_name.clone();
        async move {
            with_tenant_txn(&db, &schema_name, |txn| {
                Box::pin(async move {
                    let case_uuid = Uuid::parse_str(&case_uuid)
                        .map_err(|e| format!("invalid case uuid: {}", e))?;
                    let items = AfterSalesReworkEntity::find()
                        .filter(AfterSalesReworkColumn::CaseUuid.eq(case_uuid))
                        .order_by_desc(AfterSalesReworkColumn::InsertedAt)
                        .all(txn)
                        .await
                        .map_err(|e| format!("query after sales reworks error: {}", e))?;

                    let user_names = load_user_names(
                        txn,
                        items.iter().map(|item| item.assigned_user_uuid).collect(),
                    )
                    .await?;

                    Ok(items
                        .into_iter()
                        .map(|item| {
                            let assigned_user_name =
                                user_names.get(&item.assigned_user_uuid).cloned();
                            AfterSalesReworkMapper::to_view(item, assigned_user_name)
                        })
                        .collect())
                })
            })
            .await
        }
    }
}

async fn load_user_names(
    txn: &DatabaseTransaction,
    user_ids: Vec<Uuid>,
) -> Result<HashMap<Uuid, String>, String> {
    let user_ids = user_ids.into_iter().collect::<Vec<_>>();
    if user_ids.is_empty() {
        return Ok(HashMap::new());
    }

    let users = UserEntity::find()
        .filter(UserColumn::Uuid.is_in(user_ids))
        .all(txn)
        .await
        .map_err(|e| format!("query rework users error: {}", e))?;

    Ok(users
        .into_iter()
        .map(|user| (user.uuid, user.user_name))
        .collect())
}
