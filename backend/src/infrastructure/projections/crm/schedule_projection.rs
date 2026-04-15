use std::time::Duration;

use async_trait::async_trait;
use disintegrate::{EventListener, PersistedEvent, StreamQuery, query};
use disintegrate_postgres::{PgEventId, PgEventListener, PgEventListenerConfig};
use sea_orm::ActiveValue::Set;
use sea_orm::entity::prelude::*;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, IntoActiveModel};
use uuid::Uuid;

use crate::domain::crm::schedule::ScheduleEventEnvelope;
use crate::infrastructure::entity::{merchant, schedules};
use crate::infrastructure::event_store::schedule::event_store;
use crate::infrastructure::tenant::{is_safe_schema_name, with_tenant_txn};

pub struct ScheduleProjection {
    query: StreamQuery<PgEventId, ScheduleEventEnvelope>,
    db: DatabaseConnection,
}

impl ScheduleProjection {
    pub async fn new(db: DatabaseConnection) -> Result<Self, String> {
        ensure_existing_tenant_read_models(&db).await?;
        Ok(Self {
            query: query!(ScheduleEventEnvelope),
            db,
        })
    }
}

#[async_trait]
impl EventListener<PgEventId, ScheduleEventEnvelope> for ScheduleProjection {
    type Error = String;

    fn id(&self) -> &'static str {
        "crm.schedule.read_model"
    }

    fn query(&self) -> &StreamQuery<PgEventId, ScheduleEventEnvelope> {
        &self.query
    }

    async fn handle(
        &self,
        event: PersistedEvent<PgEventId, ScheduleEventEnvelope>,
    ) -> Result<(), Self::Error> {
        let event_id = event.id();

        match event.into_inner() {
            ScheduleEventEnvelope::ScheduleAssignmentCreated {
                tenant_schema,
                order_id,
                schedule_uuid,
                assigned_user_uuid,
                start_at,
                end_at,
                status,
                notes,
                inserted_at,
                updated_at,
            } => {
                with_tenant_txn(&self.db, &tenant_schema, |txn| {
                    Box::pin(async move {
                        let order_uuid = parse_uuid(&order_id, "order_id")?;
                        let existing = schedules::Entity::find()
                            .filter(schedules::Column::OrderId.eq(order_uuid))
                            .one(txn)
                            .await
                            .map_err(|e| format!("query schedule projection error: {}", e))?;
                        if existing.is_none() {
                            let active = schedules::ActiveModel {
                                uuid: Set(parse_uuid(&schedule_uuid, "schedule_uuid")?),
                                order_id: Set(order_uuid),
                                assigned_user_uuid: Set(parse_uuid(
                                    &assigned_user_uuid,
                                    "assigned_user_uuid",
                                )?),
                                start_at: Set(start_at),
                                end_at: Set(end_at),
                                status: Set(status),
                                notes: Set(notes),
                                inserted_at: Set(inserted_at),
                                updated_at: Set(updated_at),
                                event_id: Set(event_id),
                            };
                            active
                                .insert(txn)
                                .await
                                .map_err(|e| format!("insert schedule projection error: {}", e))?;
                        }
                        Ok(())
                    })
                })
                .await?;
            }
            ScheduleEventEnvelope::ScheduleAssignmentUpdated {
                tenant_schema,
                order_id,
                assigned_user_uuid,
                start_at,
                end_at,
                notes,
                updated_at,
            } => {
                with_tenant_txn(&self.db, &tenant_schema, |txn| {
                    Box::pin(async move {
                        let order_uuid = parse_uuid(&order_id, "order_id")?;
                        let Some(model) = schedules::Entity::find()
                            .filter(schedules::Column::OrderId.eq(order_uuid))
                            .one(txn)
                            .await
                            .map_err(|e| format!("query schedule projection error: {}", e))?
                        else {
                            return Ok(());
                        };

                        if model.event_id >= event_id {
                            return Ok(());
                        }

                        let mut active = model.into_active_model();
                        active.assigned_user_uuid =
                            Set(parse_uuid(&assigned_user_uuid, "assigned_user_uuid")?);
                        active.start_at = Set(start_at);
                        active.end_at = Set(end_at);
                        active.notes = Set(notes);
                        active.updated_at = Set(updated_at);
                        active.event_id = Set(event_id);
                        active
                            .update(txn)
                            .await
                            .map_err(|e| format!("update schedule projection error: {}", e))?;
                        Ok(())
                    })
                })
                .await?;
            }
            ScheduleEventEnvelope::ScheduleStatusChanged {
                tenant_schema,
                order_id,
                status,
                updated_at,
            } => {
                with_tenant_txn(&self.db, &tenant_schema, |txn| {
                    Box::pin(async move {
                        let order_uuid = parse_uuid(&order_id, "order_id")?;
                        let Some(model) = schedules::Entity::find()
                            .filter(schedules::Column::OrderId.eq(order_uuid))
                            .one(txn)
                            .await
                            .map_err(|e| format!("query schedule projection error: {}", e))?
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
                            .map_err(|e| format!("update schedule projection error: {}", e))?;
                        Ok(())
                    })
                })
                .await?;
            }
            ScheduleEventEnvelope::ScheduleDeleted {
                tenant_schema,
                order_id,
                deleted_at,
            } => {
                with_tenant_txn(&self.db, &tenant_schema, |txn| {
                    Box::pin(async move {
                        let order_uuid = parse_uuid(&order_id, "order_id")?;
                        let Some(model) = schedules::Entity::find()
                            .filter(schedules::Column::OrderId.eq(order_uuid))
                            .one(txn)
                            .await
                            .map_err(|e| format!("query schedule projection error: {}", e))?
                        else {
                            return Ok(());
                        };

                        if model.event_id >= event_id {
                            return Ok(());
                        }

                        let mut active = model.into_active_model();
                        active.updated_at = Set(deleted_at);
                        active
                            .delete(txn)
                            .await
                            .map_err(|e| format!("delete schedule projection error: {}", e))?;
                        Ok(())
                    })
                })
                .await?;
            }
        }

        Ok(())
    }
}

pub async fn spawn_schedule_listener(read_model_db: DatabaseConnection) -> Result<(), String> {
    let listener_event_store = event_store().await?;

    tokio::spawn(async move {
        let projection = match ScheduleProjection::new(read_model_db).await {
            Ok(projection) => projection,
            Err(err) => {
                eprintln!("failed to create schedule projection: {}", err);
                return;
            }
        };

        if let Err(err) = PgEventListener::builder(listener_event_store)
            .register_listener(
                projection,
                PgEventListenerConfig::poller(Duration::from_millis(250)).with_notifier(),
            )
            .start()
            .await
        {
            eprintln!("schedule projection listener exited with error: {}", err);
        }
    });

    Ok(())
}

async fn ensure_existing_tenant_read_models(db: &DatabaseConnection) -> Result<(), String> {
    let merchants = merchant::Entity::find()
        .all(db)
        .await
        .map_err(|e| format!("query merchants for schedule projection error: {}", e))?;

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
                "ALTER TABLE IF EXISTS schedules ADD COLUMN IF NOT EXISTS event_id BIGINT NOT NULL DEFAULT 0",
            );
            txn.execute(stmt)
                .await
                .map_err(|e| format!("ensure schedule read model event_id error: {}", e))?;
            Ok(())
        })
    })
    .await
}

fn parse_uuid(value: &str, field: &str) -> Result<Uuid, String> {
    Uuid::parse_str(value).map_err(|e| format!("invalid {} uuid {}: {}", field, value, e))
}
