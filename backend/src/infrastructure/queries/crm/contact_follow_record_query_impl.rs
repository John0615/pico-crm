use std::collections::HashMap;

use async_trait::async_trait;
use sea_orm::entity::prelude::*;
use sea_orm::{
    ColumnTrait, DatabaseConnection, DatabaseTransaction, EntityTrait, QueryFilter, QueryOrder,
};
use shared::contact::ContactFollowRecord as SharedContactFollowRecord;

use crate::domain::crm::contact::ContactFollowRecordQuery;
use crate::infrastructure::entity::contact_follow_records::{
    Column as ContactFollowRecordColumn, Entity as ContactFollowRecordEntity,
};
use crate::infrastructure::entity::users::{Column as UserColumn, Entity as UserEntity};
use crate::infrastructure::mappers::crm::contact_follow_record_mapper::ContactFollowRecordMapper;
use crate::infrastructure::tenant::with_tenant_txn;

pub struct SeaOrmContactFollowRecordQuery {
    db: DatabaseConnection,
    schema_name: String,
}

impl SeaOrmContactFollowRecordQuery {
    pub fn new(db: DatabaseConnection, schema_name: String) -> Self {
        Self { db, schema_name }
    }
}

#[async_trait]
impl ContactFollowRecordQuery for SeaOrmContactFollowRecordQuery {
    type Result = SharedContactFollowRecord;

    fn list_follow_records(
        &self,
        contact_uuid: String,
    ) -> impl std::future::Future<Output = Result<Vec<Self::Result>, String>> + Send {
        let db = self.db.clone();
        let schema_name = self.schema_name.clone();
        async move {
            with_tenant_txn(&db, &schema_name, |txn| {
                Box::pin(async move {
                    let contact_uuid = Uuid::parse_str(&contact_uuid)
                        .map_err(|e| format!("invalid contact uuid: {}", e))?;

                    let items = ContactFollowRecordEntity::find()
                        .filter(ContactFollowRecordColumn::ContactUuid.eq(contact_uuid))
                        .order_by_desc(ContactFollowRecordColumn::CreatedAt)
                        .all(txn)
                        .await
                        .map_err(|e| format!("query contact follow records error: {}", e))?;

                    let operator_names = load_operator_names(
                        txn,
                        items.iter().filter_map(|item| item.operator_uuid).collect(),
                    )
                    .await?;

                    Ok(items
                        .into_iter()
                        .map(|item| {
                            let operator_name =
                                item.operator_uuid.and_then(|uuid| operator_names.get(&uuid).cloned());
                            ContactFollowRecordMapper::to_view(item, operator_name)
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
        .map_err(|e| format!("query follow record operators error: {}", e))?;

    Ok(users
        .into_iter()
        .map(|user| (user.uuid, user.user_name))
        .collect())
}
