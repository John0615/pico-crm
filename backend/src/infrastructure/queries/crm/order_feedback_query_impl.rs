use std::collections::HashMap;

use async_trait::async_trait;
use sea_orm::entity::prelude::*;
use sea_orm::{
    ColumnTrait, DatabaseConnection, DatabaseTransaction, EntityTrait, QueryFilter, QueryOrder,
};
use shared::order::OrderFeedback as SharedOrderFeedback;

use crate::domain::crm::order::OrderFeedbackQuery;
use crate::infrastructure::entity::order_feedback::{
    Column as OrderFeedbackColumn, Entity as OrderFeedbackEntity,
};
use crate::infrastructure::entity::users::{Column as UserColumn, Entity as UserEntity};
use crate::infrastructure::mappers::crm::order_feedback_mapper::OrderFeedbackMapper;
use crate::infrastructure::tenant::with_tenant_txn;

pub struct SeaOrmOrderFeedbackQuery {
    db: DatabaseConnection,
    schema_name: String,
}

impl SeaOrmOrderFeedbackQuery {
    pub fn new(db: DatabaseConnection, schema_name: String) -> Self {
        Self { db, schema_name }
    }
}

#[async_trait]
impl OrderFeedbackQuery for SeaOrmOrderFeedbackQuery {
    type Result = SharedOrderFeedback;

    fn list_feedbacks(
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

                    let items = OrderFeedbackEntity::find()
                        .filter(OrderFeedbackColumn::OrderId.eq(order_uuid))
                        .order_by_desc(OrderFeedbackColumn::InsertedAt)
                        .all(txn)
                        .await
                        .map_err(|e| format!("query order feedbacks error: {}", e))?;

                    let user_names = load_user_names(
                        txn,
                        items.iter().filter_map(|item| item.user_uuid).collect(),
                    )
                    .await?;

                    Ok(items
                        .into_iter()
                        .map(|item| {
                            let user_name = item
                                .user_uuid
                                .and_then(|uuid| user_names.get(&uuid).cloned());
                            OrderFeedbackMapper::to_view(item, user_name)
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
        .map_err(|e| format!("query order feedback users error: {}", e))?;

    Ok(users
        .into_iter()
        .map(|user| (user.uuid, user.user_name))
        .collect())
}
