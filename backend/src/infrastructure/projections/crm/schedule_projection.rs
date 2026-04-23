use std::time::Duration;

use async_trait::async_trait;
use disintegrate::{EventListener, PersistedEvent, StreamQuery, query};
use disintegrate_postgres::{PgEventId, PgEventListener, PgEventListenerConfig};
use sea_orm::ActiveValue::Set;
use sea_orm::entity::prelude::*;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, IntoActiveModel};
use uuid::Uuid;

use crate::domain::crm::schedule::ScheduleEventEnvelope;
use crate::infrastructure::entity::schedules;
use crate::infrastructure::event_store::schedule::event_store;
use crate::infrastructure::tenant::{parse_merchant_uuid, with_shared_txn};

pub struct ScheduleProjection {
    query: StreamQuery<PgEventId, ScheduleEventEnvelope>,
    db: DatabaseConnection,
}

impl ScheduleProjection {
    pub async fn new(db: DatabaseConnection) -> Result<Self, String> {
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
                merchant_id,
                order_uuid,
                schedule_uuid,
                assigned_user_uuid,
                start_at,
                end_at,
                status,
                notes,
                inserted_at,
                updated_at,
            } => {
                let merchant_uuid = parse_merchant_uuid(&merchant_id)?;
                with_shared_txn(&self.db, |txn| {
                    Box::pin(async move {
                        let order_uuid = parse_uuid(&order_uuid, "order_uuid")?;
                        let existing = schedules::Entity::find()
                            .filter(schedules::Column::MerchantId.eq(merchant_uuid))
                            .filter(schedules::Column::OrderUuid.eq(order_uuid))
                            .one(txn)
                            .await
                            .map_err(|e| format!("query schedule projection error: {}", e))?;
                        if existing.is_none() {
                            let active = schedules::ActiveModel {
                                uuid: Set(parse_uuid(&schedule_uuid, "schedule_uuid")?),
                                merchant_id: Set(Some(merchant_uuid)),
                                order_uuid: Set(order_uuid),
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
                merchant_id,
                order_uuid,
                assigned_user_uuid,
                start_at,
                end_at,
                notes,
                updated_at,
            } => {
                let merchant_uuid = parse_merchant_uuid(&merchant_id)?;
                with_shared_txn(&self.db, |txn| {
                    Box::pin(async move {
                        let order_uuid = parse_uuid(&order_uuid, "order_uuid")?;
                        let Some(model) = schedules::Entity::find()
                            .filter(schedules::Column::MerchantId.eq(merchant_uuid))
                            .filter(schedules::Column::OrderUuid.eq(order_uuid))
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
                merchant_id,
                order_uuid,
                status,
                updated_at,
            } => {
                let merchant_uuid = parse_merchant_uuid(&merchant_id)?;
                with_shared_txn(&self.db, |txn| {
                    Box::pin(async move {
                        let order_uuid = parse_uuid(&order_uuid, "order_uuid")?;
                        let Some(model) = schedules::Entity::find()
                            .filter(schedules::Column::MerchantId.eq(merchant_uuid))
                            .filter(schedules::Column::OrderUuid.eq(order_uuid))
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
                merchant_id,
                order_uuid,
                deleted_at,
            } => {
                let merchant_uuid = parse_merchant_uuid(&merchant_id)?;
                with_shared_txn(&self.db, |txn| {
                    Box::pin(async move {
                        let order_uuid = parse_uuid(&order_uuid, "order_uuid")?;
                        let Some(model) = schedules::Entity::find()
                            .filter(schedules::Column::MerchantId.eq(merchant_uuid))
                            .filter(schedules::Column::OrderUuid.eq(order_uuid))
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
            .uninitialized()
            .register_listener(
                projection,
                PgEventListenerConfig::poller(Duration::from_millis(250))
                    .with_notifier()
                    .with_retry(|err, attempts| {
                        super::projection_listener_retry("schedule", err, attempts)
                    }),
            )
            .start()
            .await
        {
            eprintln!("schedule projection listener exited with error: {}", err);
        }
    });

    Ok(())
}

fn parse_uuid(value: &str, field: &str) -> Result<Uuid, String> {
    Uuid::parse_str(value).map_err(|e| format!("invalid {} uuid {}: {}", field, value, e))
}
