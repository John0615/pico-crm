use async_trait::async_trait;
use sea_orm::entity::prelude::*;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait};

use crate::domain::models::order::{Order, OrderAssignmentUpdate, OrderStatus, SettlementStatus};
use crate::domain::repositories::order::OrderRepository;
use crate::infrastructure::entity::orders::Entity;
use crate::infrastructure::mappers::order_mapper::OrderMapper;
use crate::infrastructure::tenant::with_tenant_txn;

pub struct SeaOrmOrderRepository {
    db: DatabaseConnection,
    schema_name: String,
}

impl SeaOrmOrderRepository {
    pub fn new(db: DatabaseConnection, schema_name: String) -> Self {
        Self { db, schema_name }
    }
}

#[async_trait]
impl OrderRepository for SeaOrmOrderRepository {
    fn create_order(&self, order: Order) -> impl std::future::Future<Output = Result<Order, String>> + Send {
        let db = self.db.clone();
        let schema_name = self.schema_name.clone();
        async move {
            with_tenant_txn(&db, &schema_name, |txn| {
                Box::pin(async move {
                    let entity = OrderMapper::to_active_entity(order);
                    let inserted = entity
                        .insert(txn)
                        .await
                        .map_err(|e| format!("create order error: {}", e))?;
                    Ok(OrderMapper::to_domain(inserted))
                })
            })
            .await
        }
    }

    fn update_order_status(
        &self,
        uuid: String,
        status: String,
    ) -> impl std::future::Future<Output = Result<Order, String>> + Send {
        let db = self.db.clone();
        let schema_name = self.schema_name.clone();
        async move {
            let status = OrderStatus::parse(&status)?;
            with_tenant_txn(&db, &schema_name, |txn| {
                let status = status;
                let uuid = uuid.clone();
                Box::pin(async move {
                    let order_uuid = Uuid::parse_str(&uuid)
                        .map_err(|e| format!("invalid order uuid: {}", e))?;
                    let original = Entity::find_by_id(order_uuid)
                        .one(txn)
                        .await
                        .map_err(|e| format!("query order error: {}", e))?
                        .ok_or_else(|| format!("order {} not found", uuid))?;

                    let active = OrderMapper::to_status_active_entity(original, status);
                    let updated = active
                        .update(txn)
                        .await
                        .map_err(|e| format!("update order status error: {}", e))?;
                    Ok(OrderMapper::to_domain(updated))
                })
            })
            .await
        }
    }

    fn update_order_assignment(
        &self,
        uuid: String,
        update: OrderAssignmentUpdate,
    ) -> impl std::future::Future<Output = Result<Order, String>> + Send {
        let db = self.db.clone();
        let schema_name = self.schema_name.clone();
        async move {
            with_tenant_txn(&db, &schema_name, |txn| {
                let uuid = uuid.clone();
                let update = update.clone();
                Box::pin(async move {
                    let order_uuid = Uuid::parse_str(&uuid)
                        .map_err(|e| format!("invalid order uuid: {}", e))?;
                    let original = Entity::find_by_id(order_uuid)
                        .one(txn)
                        .await
                        .map_err(|e| format!("query order error: {}", e))?
                        .ok_or_else(|| format!("order {} not found", uuid))?;

                    let active = OrderMapper::to_assignment_active_entity(original, update);
                    let updated = active
                        .update(txn)
                        .await
                        .map_err(|e| format!("update order assignment error: {}", e))?;
                    Ok(OrderMapper::to_domain(updated))
                })
            })
            .await
        }
    }

    fn update_order_settlement(
        &self,
        uuid: String,
        settlement_status: String,
        settlement_note: Option<String>,
    ) -> impl std::future::Future<Output = Result<Order, String>> + Send {
        let db = self.db.clone();
        let schema_name = self.schema_name.clone();
        async move {
            let settlement_status = SettlementStatus::parse(&settlement_status)?;
            with_tenant_txn(&db, &schema_name, |txn| {
                let uuid = uuid.clone();
                let settlement_status = settlement_status;
                let settlement_note = settlement_note.clone();
                Box::pin(async move {
                    let order_uuid = Uuid::parse_str(&uuid)
                        .map_err(|e| format!("invalid order uuid: {}", e))?;
                    let original = Entity::find_by_id(order_uuid)
                        .one(txn)
                        .await
                        .map_err(|e| format!("query order error: {}", e))?
                        .ok_or_else(|| format!("order {} not found", uuid))?;

                    let active = OrderMapper::to_settlement_active_entity(
                        original,
                        settlement_status,
                        settlement_note,
                    );
                    let updated = active
                        .update(txn)
                        .await
                        .map_err(|e| format!("update order settlement error: {}", e))?;
                    Ok(OrderMapper::to_domain(updated))
                })
            })
            .await
        }
    }
}
