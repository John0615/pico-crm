use async_trait::async_trait;
use chrono::Utc;
use disintegrate::{EventSourcedStateStore, LoadState, NoSnapshot};
use sea_orm::DatabaseConnection;

use crate::domain::crm::order::{
    CancelOrderDecision, CreateOrderDecision, Order, OrderAssignmentUpdate, OrderDetailsUpdate,
    OrderRepository, OrderState, OrderStatus, SettlementStatus, UpdateOrderAssignmentDecision,
    UpdateOrderDetailsDecision, UpdateOrderSettlementDecision, UpdateOrderStatusDecision,
};
use crate::infrastructure::event_store::order::event_store;

pub struct SeaOrmOrderRepository {
    _db: DatabaseConnection,
    merchant_id: String,
}

impl SeaOrmOrderRepository {
    pub fn new(db: DatabaseConnection, merchant_id: String) -> Self {
        Self {
            _db: db,
            merchant_id,
        }
    }

    async fn load_order_state(merchant_id: &str, uuid: &str) -> Result<OrderState, String> {
        let event_store = event_store().await?;
        let state_store = EventSourcedStateStore::new(event_store, NoSnapshot);
        let loaded_state = state_store
            .load(OrderState::new(merchant_id.to_string(), uuid.to_string()))
            .await
            .map_err(|e| format!("load order state error: {}", e))?;
        Ok(loaded_state.state().clone())
    }

    async fn load_order_from_events(
        merchant_id: &str,
        uuid: &str,
    ) -> Result<Option<Order>, String> {
        let state = Self::load_order_state(merchant_id, uuid).await?;
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
        operator_uuid: Option<String>,
    ) -> impl std::future::Future<Output = Result<Order, String>> + Send {
        let merchant_id = self.merchant_id.clone();
        async move {
            let order_uuid = order.uuid.clone();
            let event_store = event_store().await?;
            let decision_maker = disintegrate_postgres::decision_maker(event_store, NoSnapshot);
            decision_maker
                .make(CreateOrderDecision::new(
                    merchant_id.clone(),
                    order,
                    operator_uuid,
                ))
                .await
                .map_err(|e| format!("create order decision error: {}", e))?;

            SeaOrmOrderRepository::load_order_from_events(&merchant_id, &order_uuid)
                .await?
                .ok_or_else(|| format!("order {} not found after creation", order_uuid))
        }
    }

    fn find_order(
        &self,
        uuid: String,
    ) -> impl std::future::Future<Output = Result<Option<Order>, String>> + Send {
        let merchant_id = self.merchant_id.clone();
        async move { SeaOrmOrderRepository::load_order_from_events(&merchant_id, &uuid).await }
    }

    fn update_order_status(
        &self,
        uuid: String,
        status: String,
        operator_uuid: Option<String>,
    ) -> impl std::future::Future<Output = Result<Order, String>> + Send {
        let merchant_id = self.merchant_id.clone();
        async move {
            let next_status = OrderStatus::parse(&status)?;
            SeaOrmOrderRepository::load_order_from_events(&merchant_id, &uuid)
                .await?
                .ok_or_else(|| format!("order {} not found", uuid.clone()))?;

            let event_store = event_store().await?;
            let decision_maker = disintegrate_postgres::decision_maker(event_store, NoSnapshot);
            decision_maker
                .make(UpdateOrderStatusDecision::new(
                    merchant_id.clone(),
                    uuid.clone(),
                    next_status,
                    Utc::now(),
                    operator_uuid,
                ))
                .await
                .map_err(|e| format!("update order status decision error: {}", e))?;

            SeaOrmOrderRepository::load_order_from_events(&merchant_id, &uuid)
                .await?
                .ok_or_else(|| format!("order {} not found after status update", uuid))
        }
    }

    fn update_order_assignment(
        &self,
        uuid: String,
        update: OrderAssignmentUpdate,
        operator_uuid: Option<String>,
    ) -> impl std::future::Future<Output = Result<Order, String>> + Send {
        let merchant_id = self.merchant_id.clone();
        async move {
            SeaOrmOrderRepository::load_order_from_events(&merchant_id, &uuid)
                .await?
                .ok_or_else(|| format!("order {} not found", uuid.clone()))?;

            let event_store = event_store().await?;
            let decision_maker = disintegrate_postgres::decision_maker(event_store, NoSnapshot);
            decision_maker
                .make(UpdateOrderAssignmentDecision::new(
                    merchant_id.clone(),
                    uuid.clone(),
                    update,
                    Utc::now(),
                    operator_uuid,
                ))
                .await
                .map_err(|e| format!("update order assignment decision error: {}", e))?;

            SeaOrmOrderRepository::load_order_from_events(&merchant_id, &uuid)
                .await?
                .ok_or_else(|| format!("order {} not found after assignment update", uuid))
        }
    }

    fn update_order_details(
        &self,
        uuid: String,
        update: OrderDetailsUpdate,
        operator_uuid: Option<String>,
    ) -> impl std::future::Future<Output = Result<Order, String>> + Send {
        let merchant_id = self.merchant_id.clone();
        async move {
            SeaOrmOrderRepository::load_order_from_events(&merchant_id, &uuid)
                .await?
                .ok_or_else(|| format!("order {} not found", uuid.clone()))?;

            let event_store = event_store().await?;
            let decision_maker = disintegrate_postgres::decision_maker(event_store, NoSnapshot);
            decision_maker
                .make(UpdateOrderDetailsDecision::new(
                    merchant_id.clone(),
                    uuid.clone(),
                    update,
                    Utc::now(),
                    operator_uuid,
                ))
                .await
                .map_err(|e| format!("update order details decision error: {}", e))?;

            SeaOrmOrderRepository::load_order_from_events(&merchant_id, &uuid)
                .await?
                .ok_or_else(|| format!("order {} not found after details update", uuid))
        }
    }

    fn update_order_settlement(
        &self,
        uuid: String,
        settlement_status: String,
        settlement_note: Option<String>,
        paid_amount_cents: Option<i64>,
        payment_method: Option<String>,
        paid_at: Option<chrono::DateTime<chrono::Utc>>,
        operator_uuid: Option<String>,
    ) -> impl std::future::Future<Output = Result<Order, String>> + Send {
        let merchant_id = self.merchant_id.clone();
        async move {
            let settlement_status = SettlementStatus::parse(&settlement_status)?;
            SeaOrmOrderRepository::load_order_from_events(&merchant_id, &uuid)
                .await?
                .ok_or_else(|| format!("order {} not found", uuid.clone()))?;

            let event_store = event_store().await?;
            let decision_maker = disintegrate_postgres::decision_maker(event_store, NoSnapshot);
            decision_maker
                .make(UpdateOrderSettlementDecision::new(
                    merchant_id.clone(),
                    uuid.clone(),
                    settlement_status,
                    settlement_note,
                    paid_amount_cents,
                    payment_method,
                    paid_at,
                    Utc::now(),
                    operator_uuid,
                ))
                .await
                .map_err(|e| format!("update order settlement decision error: {}", e))?;

            SeaOrmOrderRepository::load_order_from_events(&merchant_id, &uuid)
                .await?
                .ok_or_else(|| format!("order {} not found after settlement update", uuid))
        }
    }

    fn cancel_order(
        &self,
        uuid: String,
        reason: String,
        operator_uuid: Option<String>,
    ) -> impl std::future::Future<Output = Result<Order, String>> + Send {
        let merchant_id = self.merchant_id.clone();
        async move {
            SeaOrmOrderRepository::load_order_from_events(&merchant_id, &uuid)
                .await?
                .ok_or_else(|| format!("order {} not found", uuid.clone()))?;

            let event_store = event_store().await?;
            let decision_maker = disintegrate_postgres::decision_maker(event_store, NoSnapshot);
            decision_maker
                .make(CancelOrderDecision::new(
                    merchant_id.clone(),
                    uuid.clone(),
                    reason,
                    Utc::now(),
                    operator_uuid,
                ))
                .await
                .map_err(|e| format!("cancel order decision error: {}", e))?;

            SeaOrmOrderRepository::load_order_from_events(&merchant_id, &uuid)
                .await?
                .ok_or_else(|| format!("order {} not found after cancellation", uuid))
        }
    }
}
