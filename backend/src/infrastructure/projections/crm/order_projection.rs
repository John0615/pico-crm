use std::time::Duration;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use disintegrate::{query, EventListener, PersistedEvent, StreamQuery};
use disintegrate_postgres::{PgEventId, PgEventListener, PgEventListenerConfig};
use sea_orm::entity::prelude::*;
use sea_orm::ActiveValue::Set;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, IntoActiveModel};
use serde_json::Value;
use uuid::Uuid;

use crate::domain::crm::order::OrderEventEnvelope;
use crate::infrastructure::entity::{merchant, order_change_logs, orders};
use crate::infrastructure::event_store::order::event_store;
use crate::infrastructure::tenant::{is_safe_schema_name, with_tenant_txn};

pub struct OrderProjection {
    query: StreamQuery<PgEventId, OrderEventEnvelope>,
    db: DatabaseConnection,
}

impl OrderProjection {
    pub async fn new(db: DatabaseConnection) -> Result<Self, String> {
        ensure_existing_tenant_read_models(&db).await?;
        Ok(Self {
            query: query!(OrderEventEnvelope),
            db,
        })
    }
}

#[async_trait]
impl EventListener<PgEventId, OrderEventEnvelope> for OrderProjection {
    type Error = String;

    fn id(&self) -> &'static str {
        "crm.order.read_model"
    }

    fn query(&self) -> &StreamQuery<PgEventId, OrderEventEnvelope> {
        &self.query
    }

    async fn handle(
        &self,
        event: PersistedEvent<PgEventId, OrderEventEnvelope>,
    ) -> Result<(), Self::Error> {
        let event_id = event.id();

        match event.into_inner() {
            OrderEventEnvelope::OrderCreated {
                tenant_schema,
                order_uuid,
                operator_uuid,
                request_id,
                customer_uuid,
                scheduled_start_at,
                scheduled_end_at,
                status,
                cancellation_reason,
                completed_at,
                settlement_status,
                amount_cents,
                notes,
                dispatch_note,
                settlement_note,
                inserted_at,
                updated_at,
            } => {
                with_tenant_txn(&self.db, &tenant_schema, |txn| {
                    Box::pin(async move {
                        let order_uuid = parse_uuid(&order_uuid, "order_uuid")?;
                        let existing = orders::Entity::find_by_id(order_uuid)
                            .one(txn)
                            .await
                            .map_err(|e| format!("query order projection error: {}", e))?;
                        if existing.is_none() {
                            let active = orders::ActiveModel {
                                uuid: Set(order_uuid),
                                customer_uuid: Set(parse_optional_uuid(
                                    customer_uuid.as_deref(),
                                    "customer_uuid",
                                )?),
                                status: Set(status),
                                cancellation_reason: Set(cancellation_reason),
                                completed_at: Set(completed_at),
                                amount_cents: Set(amount_cents),
                                notes: Set(notes),
                                request_id: Set(parse_optional_uuid(
                                    request_id.as_deref(),
                                    "request_id",
                                )?),
                                scheduled_start_at: Set(scheduled_start_at),
                                scheduled_end_at: Set(scheduled_end_at),
                                dispatch_note: Set(dispatch_note),
                                settlement_status: Set(settlement_status),
                                settlement_note: Set(settlement_note),
                                inserted_at: Set(inserted_at),
                                updated_at: Set(updated_at),
                                event_id: Set(event_id),
                            };
                            let created = active
                                .insert(txn)
                                .await
                                .map_err(|e| format!("insert order projection error: {}", e))?;
                            insert_change_log(
                                txn,
                                created.uuid,
                                "created",
                                parse_optional_uuid(operator_uuid.as_deref(), "operator_uuid")?,
                                None,
                                Some(snapshot_order_model(&created)),
                                created.inserted_at,
                            )
                            .await?;
                        }
                        Ok(())
                    })
                })
                .await?;
            }
            OrderEventEnvelope::OrderDetailsUpdated {
                tenant_schema,
                order_uuid,
                operator_uuid,
                customer_uuid,
                amount_cents,
                notes,
                updated_at,
            } => {
                with_tenant_txn(&self.db, &tenant_schema, |txn| {
                    Box::pin(async move {
                        let order_uuid = parse_uuid(&order_uuid, "order_uuid")?;
                        let Some(model) = orders::Entity::find_by_id(order_uuid)
                            .one(txn)
                            .await
                            .map_err(|e| format!("query order projection error: {}", e))?
                        else {
                            return Ok(());
                        };
                        if model.event_id >= event_id {
                            return Ok(());
                        }

                        let before = snapshot_order_model(&model);
                        let mut active = model.into_active_model();
                        active.customer_uuid = Set(parse_optional_uuid(
                            customer_uuid.as_deref(),
                            "customer_uuid",
                        )?);
                        active.amount_cents = Set(amount_cents);
                        active.notes = Set(notes);
                        active.updated_at = Set(updated_at);
                        active.event_id = Set(event_id);
                        let updated = active
                            .update(txn)
                            .await
                            .map_err(|e| format!("update order projection error: {}", e))?;
                        insert_change_log(
                            txn,
                            updated.uuid,
                            "details_updated",
                            parse_optional_uuid(operator_uuid.as_deref(), "operator_uuid")?,
                            Some(before),
                            Some(snapshot_order_model(&updated)),
                            updated_at,
                        )
                        .await?;
                        Ok(())
                    })
                })
                .await?;
            }
            OrderEventEnvelope::OrderStatusChanged {
                tenant_schema,
                order_uuid,
                operator_uuid,
                status,
                completed_at,
                updated_at,
            } => {
                with_tenant_txn(&self.db, &tenant_schema, |txn| {
                    Box::pin(async move {
                        let order_uuid = parse_uuid(&order_uuid, "order_uuid")?;
                        let Some(model) = orders::Entity::find_by_id(order_uuid)
                            .one(txn)
                            .await
                            .map_err(|e| format!("query order projection error: {}", e))?
                        else {
                            return Ok(());
                        };
                        if model.event_id >= event_id {
                            return Ok(());
                        }

                        let before = snapshot_order_model(&model);
                        let mut active = model.into_active_model();
                        active.status = Set(status);
                        active.completed_at = Set(completed_at);
                        active.updated_at = Set(updated_at);
                        active.event_id = Set(event_id);
                        let updated = active
                            .update(txn)
                            .await
                            .map_err(|e| format!("update order projection error: {}", e))?;
                        insert_change_log(
                            txn,
                            updated.uuid,
                            "status_changed",
                            parse_optional_uuid(operator_uuid.as_deref(), "operator_uuid")?,
                            Some(before),
                            Some(snapshot_order_model(&updated)),
                            updated_at,
                        )
                        .await?;
                        Ok(())
                    })
                })
                .await?;
            }
            OrderEventEnvelope::OrderCancelled {
                tenant_schema,
                order_uuid,
                operator_uuid,
                cancellation_reason,
                updated_at,
            } => {
                with_tenant_txn(&self.db, &tenant_schema, |txn| {
                    Box::pin(async move {
                        let order_uuid = parse_uuid(&order_uuid, "order_uuid")?;
                        let Some(model) = orders::Entity::find_by_id(order_uuid)
                            .one(txn)
                            .await
                            .map_err(|e| format!("query order projection error: {}", e))?
                        else {
                            return Ok(());
                        };
                        if model.event_id >= event_id {
                            return Ok(());
                        }

                        let before = snapshot_order_model(&model);
                        let mut active = model.into_active_model();
                        active.status = Set("cancelled".to_string());
                        active.cancellation_reason = Set(Some(cancellation_reason));
                        active.completed_at = Set(None);
                        active.updated_at = Set(updated_at);
                        active.event_id = Set(event_id);
                        let updated = active
                            .update(txn)
                            .await
                            .map_err(|e| format!("update order projection error: {}", e))?;
                        insert_change_log(
                            txn,
                            updated.uuid,
                            "cancelled",
                            parse_optional_uuid(operator_uuid.as_deref(), "operator_uuid")?,
                            Some(before),
                            Some(snapshot_order_model(&updated)),
                            updated_at,
                        )
                        .await?;
                        Ok(())
                    })
                })
                .await?;
            }
            OrderEventEnvelope::OrderAssignmentUpdated {
                tenant_schema,
                order_uuid,
                operator_uuid,
                scheduled_start_at,
                scheduled_end_at,
                dispatch_note,
                updated_at,
            } => {
                with_tenant_txn(&self.db, &tenant_schema, |txn| {
                    Box::pin(async move {
                        let order_uuid = parse_uuid(&order_uuid, "order_uuid")?;
                        let Some(model) = orders::Entity::find_by_id(order_uuid)
                            .one(txn)
                            .await
                            .map_err(|e| format!("query order projection error: {}", e))?
                        else {
                            return Ok(());
                        };
                        if model.event_id >= event_id {
                            return Ok(());
                        }

                        let before = snapshot_order_model(&model);
                        let mut active = model.into_active_model();
                        active.scheduled_start_at = Set(scheduled_start_at);
                        active.scheduled_end_at = Set(scheduled_end_at);
                        active.dispatch_note = Set(dispatch_note);
                        active.updated_at = Set(updated_at);
                        active.event_id = Set(event_id);
                        let updated = active
                            .update(txn)
                            .await
                            .map_err(|e| format!("update order projection error: {}", e))?;
                        insert_change_log(
                            txn,
                            updated.uuid,
                            "assignment_updated",
                            parse_optional_uuid(operator_uuid.as_deref(), "operator_uuid")?,
                            Some(before),
                            Some(snapshot_order_model(&updated)),
                            updated_at,
                        )
                        .await?;
                        Ok(())
                    })
                })
                .await?;
            }
            OrderEventEnvelope::OrderSettlementUpdated {
                tenant_schema,
                order_uuid,
                operator_uuid,
                settlement_status,
                settlement_note,
                updated_at,
            } => {
                with_tenant_txn(&self.db, &tenant_schema, |txn| {
                    Box::pin(async move {
                        let order_uuid = parse_uuid(&order_uuid, "order_uuid")?;
                        let Some(model) = orders::Entity::find_by_id(order_uuid)
                            .one(txn)
                            .await
                            .map_err(|e| format!("query order projection error: {}", e))?
                        else {
                            return Ok(());
                        };
                        if model.event_id >= event_id {
                            return Ok(());
                        }

                        let before = snapshot_order_model(&model);
                        let mut active = model.into_active_model();
                        active.settlement_status = Set(settlement_status);
                        active.settlement_note = Set(settlement_note);
                        active.updated_at = Set(updated_at);
                        active.event_id = Set(event_id);
                        let updated = active
                            .update(txn)
                            .await
                            .map_err(|e| format!("update order projection error: {}", e))?;
                        insert_change_log(
                            txn,
                            updated.uuid,
                            "settlement_updated",
                            parse_optional_uuid(operator_uuid.as_deref(), "operator_uuid")?,
                            Some(before),
                            Some(snapshot_order_model(&updated)),
                            updated_at,
                        )
                        .await?;
                        Ok(())
                    })
                })
                .await?;
            }
        }

        Ok(())
    }
}

pub async fn spawn_order_listener(read_model_db: DatabaseConnection) -> Result<(), String> {
    let listener_event_store = event_store().await?;
    let projection = OrderProjection::new(read_model_db.clone()).await?;

    tokio::spawn(async move {
        if let Err(err) = PgEventListener::builder(listener_event_store)
            .uninitialized()
            .register_listener(
                projection,
                PgEventListenerConfig::poller(Duration::from_millis(250))
                    .with_notifier()
                    .with_retry(|err, attempts| {
                        super::projection_listener_retry("order", err, attempts)
                    }),
            )
            .start()
            .await
        {
            eprintln!("order projection listener exited with error: {}", err);
        }
    });

    Ok(())
}

async fn ensure_existing_tenant_read_models(db: &DatabaseConnection) -> Result<(), String> {
    let merchants = merchant::Entity::find()
        .all(db)
        .await
        .map_err(|e| format!("query merchants for order projection error: {}", e))?;

    for merchant in merchants {
        ensure_tenant_read_model(db, &merchant.schema_name).await?;
    }

    Ok(())
}

async fn ensure_tenant_read_model(
    db: &DatabaseConnection,
    schema_name: &str,
) -> Result<(), String> {
    if !is_safe_schema_name(schema_name) {
        return Err(format!("invalid tenant schema name: {}", schema_name));
    }

    with_tenant_txn(db, schema_name, |txn| {
        Box::pin(async move {
            use sea_orm::{ConnectionTrait, DatabaseBackend, Statement};

            let statements = [
                "ALTER TABLE IF EXISTS orders ADD COLUMN IF NOT EXISTS event_id BIGINT NOT NULL DEFAULT 0".to_string(),
                "ALTER TABLE IF EXISTS orders ADD COLUMN IF NOT EXISTS cancellation_reason TEXT NULL".to_string(),
                "ALTER TABLE IF EXISTS orders ADD COLUMN IF NOT EXISTS completed_at TIMESTAMPTZ NULL".to_string(),
                "CREATE TABLE IF NOT EXISTS order_change_logs (
                    uuid UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                    order_uuid UUID NOT NULL,
                    action VARCHAR NOT NULL,
                    operator_uuid UUID NULL,
                    before_data JSONB NULL,
                    after_data JSONB NULL,
                    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
                )"
                .to_string(),
            ];

            for sql in statements {
                let stmt = Statement::from_string(DatabaseBackend::Postgres, sql);
                txn.execute(stmt)
                    .await
                    .map_err(|e| format!("ensure order read model error: {}", e))?;
            }

            Ok(())
        })
    })
    .await
}

async fn insert_change_log<C>(
    txn: &C,
    order_uuid: Uuid,
    action: &str,
    operator_uuid: Option<Uuid>,
    before_data: Option<Value>,
    after_data: Option<Value>,
    created_at: DateTime<Utc>,
) -> Result<(), String>
where
    C: sea_orm::ConnectionTrait,
{
    let active = order_change_logs::ActiveModel {
        uuid: Set(Uuid::new_v4()),
        order_uuid: Set(order_uuid),
        action: Set(action.to_string()),
        operator_uuid: Set(operator_uuid),
        before_data: Set(before_data.map(Json::from)),
        after_data: Set(after_data.map(Json::from)),
        created_at: Set(created_at),
    };
    active
        .insert(txn)
        .await
        .map_err(|e| format!("insert order change log error: {}", e))?;
    Ok(())
}

fn snapshot_order_model(model: &orders::Model) -> Value {
    serde_json::json!({
        "uuid": model.uuid.to_string(),
        "request_id": model.request_id.map(|value| value.to_string()),
        "customer_uuid": model.customer_uuid.map(|value| value.to_string()),
        "status": model.status,
        "cancellation_reason": model.cancellation_reason,
        "completed_at": model.completed_at.map(|value| value.to_rfc3339()),
        "settlement_status": model.settlement_status,
        "amount_cents": model.amount_cents,
        "notes": model.notes,
        "dispatch_note": model.dispatch_note,
        "settlement_note": model.settlement_note,
        "scheduled_start_at": model.scheduled_start_at.map(|value| value.to_rfc3339()),
        "scheduled_end_at": model.scheduled_end_at.map(|value| value.to_rfc3339()),
        "inserted_at": model.inserted_at.to_rfc3339(),
        "updated_at": model.updated_at.to_rfc3339(),
        "event_id": model.event_id,
    })
}

fn parse_uuid(value: &str, field: &str) -> Result<Uuid, String> {
    Uuid::parse_str(value).map_err(|e| format!("invalid {} uuid {}: {}", field, value, e))
}

fn parse_optional_uuid(value: Option<&str>, field: &str) -> Result<Option<Uuid>, String> {
    value.map(|raw| parse_uuid(raw, field)).transpose()
}
