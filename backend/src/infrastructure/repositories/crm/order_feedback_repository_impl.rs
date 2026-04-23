use async_trait::async_trait;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use uuid::Uuid;

use crate::domain::crm::order::{CreateOrderFeedback, OrderFeedback, OrderFeedbackRepository};
use crate::infrastructure::entity::order_feedback::{
    Column as OrderFeedbackColumn, Entity as OrderFeedbackEntity,
};
use crate::infrastructure::mappers::crm::order_feedback_mapper::OrderFeedbackMapper;
use crate::infrastructure::tenant::{parse_merchant_uuid, with_shared_txn};

pub struct SeaOrmOrderFeedbackRepository {
    db: DatabaseConnection,
    merchant_id: String,
}

impl SeaOrmOrderFeedbackRepository {
    pub fn new(db: DatabaseConnection, merchant_id: String) -> Self {
        Self { db, merchant_id }
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
        let merchant_id = self.merchant_id.clone();
        async move {
            with_shared_txn(&db, |txn| {
                let merchant_id = merchant_id.clone();
                Box::pin(async move {
                    let merchant_uuid = parse_merchant_uuid(&merchant_id)?;
                    let order_uuid = Uuid::parse_str(&order_uuid)
                        .map_err(|e| format!("invalid order_uuid: {}", e))?;
                    let user_uuid = Uuid::parse_str(&user_uuid)
                        .map_err(|e| format!("invalid user_uuid: {}", e))?;

                    let exists = OrderFeedbackEntity::find()
                        .filter(OrderFeedbackColumn::MerchantId.eq(merchant_uuid))
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
        let merchant_id = self.merchant_id.clone();
        async move {
            with_shared_txn(&db, |txn| {
                let merchant_id = merchant_id.clone();
                Box::pin(async move {
                    let merchant_uuid = parse_merchant_uuid(&merchant_id)?;
                    let mut active = OrderFeedbackMapper::to_active_entity(feedback)?;
                    active.merchant_id = sea_orm::ActiveValue::Set(Some(merchant_uuid));
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
