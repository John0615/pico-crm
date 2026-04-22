use async_trait::async_trait;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use uuid::Uuid;

use crate::domain::crm::order::{CreateOrderFeedback, OrderFeedback, OrderFeedbackRepository};
use crate::infrastructure::entity::order_feedback::{
    Column as OrderFeedbackColumn, Entity as OrderFeedbackEntity,
};
use crate::infrastructure::mappers::crm::order_feedback_mapper::OrderFeedbackMapper;
use crate::infrastructure::tenant::with_tenant_txn;

pub struct SeaOrmOrderFeedbackRepository {
    db: DatabaseConnection,
    schema_name: String,
}

impl SeaOrmOrderFeedbackRepository {
    pub fn new(db: DatabaseConnection, schema_name: String) -> Self {
        Self { db, schema_name }
    }
}

#[async_trait]
impl OrderFeedbackRepository for SeaOrmOrderFeedbackRepository {
    fn has_feedback_for_order_user(
        &self,
        order_uuid: String,
        user_uuid: String,
    ) -> impl std::future::Future<Output = Result<bool, String>> + Send {
        let db = self.db.clone();
        let schema_name = self.schema_name.clone();
        async move {
            with_tenant_txn(&db, &schema_name, |txn| {
                Box::pin(async move {
                    let order_uuid = Uuid::parse_str(&order_uuid)
                        .map_err(|e| format!("invalid order_uuid: {}", e))?;
                    let user_uuid = Uuid::parse_str(&user_uuid)
                        .map_err(|e| format!("invalid user_uuid: {}", e))?;

                    let exists = OrderFeedbackEntity::find()
                        .filter(OrderFeedbackColumn::OrderId.eq(order_uuid))
                        .filter(OrderFeedbackColumn::UserUuid.eq(user_uuid))
                        .one(txn)
                        .await
                        .map_err(|e| format!("query order feedback exists error: {}", e))?
                        .is_some();

                    Ok(exists)
                })
            })
            .await
        }
    }

    fn create_feedback(
        &self,
        feedback: CreateOrderFeedback,
    ) -> impl std::future::Future<Output = Result<OrderFeedback, String>> + Send {
        let db = self.db.clone();
        let schema_name = self.schema_name.clone();
        async move {
            with_tenant_txn(&db, &schema_name, |txn| {
                Box::pin(async move {
                    let active = OrderFeedbackMapper::to_active_entity(feedback)?;
                    let created = active.insert(txn).await.map_err(|e| {
                        let message = e.to_string();
                        if message.contains("ux_order_feedback_order_user") {
                            "当前服务人员已提交过该订单反馈".to_string()
                        } else {
                            format!("create order feedback error: {}", e)
                        }
                    })?;

                    Ok(OrderFeedbackMapper::to_domain(created, None))
                })
            })
            .await
        }
    }
}
