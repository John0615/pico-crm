use std::time::Duration;

use async_trait::async_trait;
use disintegrate::{EventListener, PersistedEvent, StreamQuery, query};
use disintegrate_postgres::{PgEventId, PgEventListener, PgEventListenerConfig};
use sea_orm::ActiveValue::Set;
use sea_orm::entity::prelude::*;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, IntoActiveModel};
use uuid::Uuid;

use crate::domain::crm::order::OrderEventEnvelope;
use crate::infrastructure::entity::{merchant, orders};
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
                request_id,
                customer_uuid,
                scheduled_start_at,
                scheduled_end_at,
                status,
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
                            active
                                .insert(txn)
                                .await
                                .map_err(|e| format!("insert order projection error: {}", e))?;
                        }
                        Ok(())
                    })
                })
                .await?;
            }
            OrderEventEnvelope::OrderStatusChanged {
                tenant_schema,
                order_uuid,
                status,
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

                        let mut active = model.into_active_model();
                        active.status = Set(status);
                        active.updated_at = Set(updated_at);
                        active.event_id = Set(event_id);
                        active
                            .update(txn)
                            .await
                            .map_err(|e| format!("update order projection error: {}", e))?;
                        Ok(())
                    })
                })
                .await?;
            }
            OrderEventEnvelope::OrderAssignmentUpdated {
                tenant_schema,
                order_uuid,
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

                        let mut active = model.into_active_model();
                        active.scheduled_start_at = Set(scheduled_start_at);
                        active.scheduled_end_at = Set(scheduled_end_at);
                        active.dispatch_note = Set(dispatch_note);
                        active.updated_at = Set(updated_at);
                        active.event_id = Set(event_id);
                        active
                            .update(txn)
                            .await
                            .map_err(|e| format!("update order projection error: {}", e))?;
                        Ok(())
                    })
                })
                .await?;
            }
            OrderEventEnvelope::OrderSettlementUpdated {
                tenant_schema,
                order_uuid,
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

                        let mut active = model.into_active_model();
                        active.settlement_status = Set(settlement_status);
                        active.settlement_note = Set(settlement_note);
                        active.updated_at = Set(updated_at);
                        active.event_id = Set(event_id);
                        active
                            .update(txn)
                            .await
                            .map_err(|e| format!("update order projection error: {}", e))?;
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
            .register_listener(
                projection,
                PgEventListenerConfig::poller(Duration::from_millis(250)).with_notifier(),
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

            let stmt = Statement::from_string(
                DatabaseBackend::Postgres,
                "ALTER TABLE IF EXISTS orders ADD COLUMN IF NOT EXISTS event_id BIGINT NOT NULL DEFAULT 0",
            );
            txn.execute(stmt)
                .await
                .map_err(|e| format!("ensure order read model event_id error: {}", e))?;
            Ok(())
        })
    })
    .await
}

fn parse_uuid(value: &str, field: &str) -> Result<Uuid, String> {
    Uuid::parse_str(value).map_err(|e| format!("invalid {} uuid {}: {}", field, value, e))
}

fn parse_optional_uuid(value: Option<&str>, field: &str) -> Result<Option<Uuid>, String> {
    value.map(|raw| parse_uuid(raw, field)).transpose()
}
