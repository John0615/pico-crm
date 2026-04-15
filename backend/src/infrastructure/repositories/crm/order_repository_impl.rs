use async_trait::async_trait;
use chrono::Utc;
use disintegrate::{EventSourcedStateStore, LoadState, NoSnapshot};
use sea_orm::DatabaseConnection;

use crate::domain::crm::order::{
    CreateOrderDecision, Order, OrderAssignmentUpdate, OrderRepository, OrderState, OrderStatus,
    SettlementStatus, UpdateOrderAssignmentDecision, UpdateOrderSettlementDecision,
    UpdateOrderStatusDecision,
};
use crate::infrastructure::event_store::order::event_store;

pub struct SeaOrmOrderRepository {
    _db: DatabaseConnection,
    schema_name: String,
}

impl SeaOrmOrderRepository {
    pub fn new(db: DatabaseConnection, schema_name: String) -> Self {
        Self {
            _db: db,
            schema_name,
        }
    }

    async fn load_order_state(schema_name: &str, uuid: &str) -> Result<OrderState, String> {
        let event_store = event_store().await?;
        let state_store = EventSourcedStateStore::new(event_store, NoSnapshot);
        let loaded_state = state_store
            .load(OrderState::new(schema_name.to_string(), uuid.to_string()))
            .await
            .map_err(|e| format!("load order state error: {}", e))?;
        Ok(loaded_state.state().clone())
    }

    async fn load_order_from_events(
        schema_name: &str,
        uuid: &str,
    ) -> Result<Option<Order>, String> {
        let state = Self::load_order_state(schema_name, uuid).await?;
        if !state.exists {
            return Ok(None);
        }
        Ok(Some(state.to_domain()?))
    }
}

#[async_trait]
impl OrderRepository for SeaOrmOrderRepository {
    fn create_order(
        &self,
        order: Order,
    ) -> impl std::future::Future<Output = Result<Order, String>> + Send {
        let schema_name = self.schema_name.clone();
        async move {
            let order_uuid = order.uuid.clone();
            let event_store = event_store().await?;
            let decision_maker = disintegrate_postgres::decision_maker(event_store, NoSnapshot);
            decision_maker
                .make(CreateOrderDecision::new(schema_name.clone(), order))
                .await
                .map_err(|e| format!("create order decision error: {}", e))?;

            SeaOrmOrderRepository::load_order_from_events(&schema_name, &order_uuid)
                .await?
                .ok_or_else(|| format!("order {} not found after creation", order_uuid))
        }
    }

    fn find_order(
        &self,
        uuid: String,
    ) -> impl std::future::Future<Output = Result<Option<Order>, String>> + Send {
        let schema_name = self.schema_name.clone();
        async move { SeaOrmOrderRepository::load_order_from_events(&schema_name, &uuid).await }
    }

    fn update_order_status(
        &self,
        uuid: String,
        status: String,
    ) -> impl std::future::Future<Output = Result<Order, String>> + Send {
        let schema_name = self.schema_name.clone();
        async move {
            let next_status = OrderStatus::parse(&status)?;
            SeaOrmOrderRepository::load_order_from_events(&schema_name, &uuid)
                .await?
                .ok_or_else(|| format!("order {} not found", uuid.clone()))?;

            let event_store = event_store().await?;
            let decision_maker = disintegrate_postgres::decision_maker(event_store, NoSnapshot);
            decision_maker
                .make(UpdateOrderStatusDecision::new(
                    schema_name.clone(),
                    uuid.clone(),
                    next_status,
                    Utc::now(),
                ))
                .await
                .map_err(|e| format!("update order status decision error: {}", e))?;

            SeaOrmOrderRepository::load_order_from_events(&schema_name, &uuid)
                .await?
                .ok_or_else(|| format!("order {} not found after status update", uuid))
        }
    }

    fn update_order_assignment(
        &self,
        uuid: String,
        update: OrderAssignmentUpdate,
    ) -> impl std::future::Future<Output = Result<Order, String>> + Send {
        let schema_name = self.schema_name.clone();
        async move {
            SeaOrmOrderRepository::load_order_from_events(&schema_name, &uuid)
                .await?
                .ok_or_else(|| format!("order {} not found", uuid.clone()))?;

            let event_store = event_store().await?;
            let decision_maker = disintegrate_postgres::decision_maker(event_store, NoSnapshot);
            decision_maker
                .make(UpdateOrderAssignmentDecision::new(
                    schema_name.clone(),
                    uuid.clone(),
                    update,
                    Utc::now(),
                ))
                .await
                .map_err(|e| format!("update order assignment decision error: {}", e))?;

            SeaOrmOrderRepository::load_order_from_events(&schema_name, &uuid)
                .await?
                .ok_or_else(|| format!("order {} not found after assignment update", uuid))
        }
    }

    fn update_order_settlement(
        &self,
        uuid: String,
        settlement_status: String,
        settlement_note: Option<String>,
    ) -> impl std::future::Future<Output = Result<Order, String>> + Send {
        let schema_name = self.schema_name.clone();
        async move {
            let settlement_status = SettlementStatus::parse(&settlement_status)?;
            SeaOrmOrderRepository::load_order_from_events(&schema_name, &uuid)
                .await?
                .ok_or_else(|| format!("order {} not found", uuid.clone()))?;

            let event_store = event_store().await?;
            let decision_maker = disintegrate_postgres::decision_maker(event_store, NoSnapshot);
            decision_maker
                .make(UpdateOrderSettlementDecision::new(
                    schema_name.clone(),
                    uuid.clone(),
                    settlement_status,
                    settlement_note,
                    Utc::now(),
                ))
                .await
                .map_err(|e| format!("update order settlement decision error: {}", e))?;

            SeaOrmOrderRepository::load_order_from_events(&schema_name, &uuid)
                .await?
                .ok_or_else(|| format!("order {} not found after settlement update", uuid))
        }
    }
}
