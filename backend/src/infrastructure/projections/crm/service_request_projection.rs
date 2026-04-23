use std::time::Duration;

use async_trait::async_trait;
use disintegrate::{EventListener, PersistedEvent, StreamQuery, query};
use disintegrate_postgres::{PgEventId, PgEventListener, PgEventListenerConfig};
use sea_orm::ActiveValue::Set;
use sea_orm::entity::prelude::*;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, IntoActiveModel};
use uuid::Uuid;

use crate::domain::crm::service_request::ServiceRequestEventEnvelope;
use crate::infrastructure::entity::service_requests;
use crate::infrastructure::event_store::service_request::event_store;
use crate::infrastructure::tenant::{parse_merchant_uuid, with_shared_txn};

pub struct ServiceRequestProjection {
    query: StreamQuery<PgEventId, ServiceRequestEventEnvelope>,
    db: DatabaseConnection,
}

impl ServiceRequestProjection {
    pub async fn new(db: DatabaseConnection) -> Result<Self, String> {
        Ok(Self {
            query: query!(ServiceRequestEventEnvelope),
            db,
        })
    }
}

#[async_trait]
impl EventListener<PgEventId, ServiceRequestEventEnvelope> for ServiceRequestProjection {
    type Error = String;

    fn id(&self) -> &'static str {
        "crm.service_request.read_model"
    }

    fn query(&self) -> &StreamQuery<PgEventId, ServiceRequestEventEnvelope> {
        &self.query
    }

    async fn handle(
        &self,
        event: PersistedEvent<PgEventId, ServiceRequestEventEnvelope>,
    ) -> Result<(), Self::Error> {
        let event_id = event.id();

        match event.into_inner() {
            ServiceRequestEventEnvelope::ServiceRequestCreated {
                merchant_id,
                request_uuid,
                customer_uuid,
                creator_uuid,
                service_catalog_uuid,
                service_content,
                appointment_start_at,
                appointment_end_at,
                status,
                source,
                notes,
                inserted_at,
                updated_at,
            } => {
                let merchant_uuid = parse_merchant_uuid(&merchant_id)?;
                with_shared_txn(&self.db, |txn| {
                    Box::pin(async move {
                        let request_uuid = parse_uuid(&request_uuid, "request_uuid")?;
                        let existing = service_requests::Entity::find()
                            .filter(service_requests::Column::MerchantId.eq(merchant_uuid))
                            .filter(service_requests::Column::Uuid.eq(request_uuid))
                            .one(txn)
                            .await
                            .map_err(|e| {
                                format!("query service request projection error: {}", e)
                            })?;
                        if existing.is_none() {
                            let active = service_requests::ActiveModel {
                                uuid: Set(request_uuid),
                                merchant_id: Set(Some(merchant_uuid)),
                                customer_uuid: Set(parse_uuid(&customer_uuid, "customer_uuid")?),
                                creator_uuid: Set(parse_uuid(&creator_uuid, "creator_uuid")?),
                                service_catalog_uuid: Set(parse_optional_uuid(
                                    service_catalog_uuid.as_deref(),
                                    "service_catalog_uuid",
                                )?),
                                service_content: Set(service_content),
                                appointment_start_at: Set(appointment_start_at),
                                appointment_end_at: Set(appointment_end_at),
                                status: Set(status),
                                source: Set(source),
                                notes: Set(notes),
                                inserted_at: Set(inserted_at),
                                updated_at: Set(updated_at),
                                event_id: Set(event_id),
                            };
                            active.insert(txn).await.map_err(|e| {
                                format!("insert service request projection error: {}", e)
                            })?;
                        }
                        Ok(())
                    })
                })
                .await?;
            }
            ServiceRequestEventEnvelope::ServiceRequestDetailsUpdated {
                merchant_id,
                request_uuid,
                service_catalog_uuid,
                service_content,
                appointment_start_at,
                appointment_end_at,
                notes,
                updated_at,
            } => {
                let merchant_uuid = parse_merchant_uuid(&merchant_id)?;
                with_shared_txn(&self.db, |txn| {
                    Box::pin(async move {
                        let request_uuid = parse_uuid(&request_uuid, "request_uuid")?;
                        let Some(model) = service_requests::Entity::find()
                            .filter(service_requests::Column::MerchantId.eq(merchant_uuid))
                            .filter(service_requests::Column::Uuid.eq(request_uuid))
                            .one(txn)
                            .await
                            .map_err(|e| {
                                format!("query service request projection error: {}", e)
                            })?
                        else {
                            return Ok(());
                        };

                        if model.event_id >= event_id {
                            return Ok(());
                        }

                        let mut active = model.into_active_model();
                        active.service_catalog_uuid = Set(parse_optional_uuid(
                            service_catalog_uuid.as_deref(),
                            "service_catalog_uuid",
                        )?);
                        active.service_content = Set(service_content);
                        active.appointment_start_at = Set(appointment_start_at);
                        active.appointment_end_at = Set(appointment_end_at);
                        active.notes = Set(notes);
                        active.updated_at = Set(updated_at);
                        active.event_id = Set(event_id);
                        active.update(txn).await.map_err(|e| {
                            format!("update service request projection error: {}", e)
                        })?;
                        Ok(())
                    })
                })
                .await?;
            }
            ServiceRequestEventEnvelope::ServiceRequestStatusChanged {
                merchant_id,
                request_uuid,
                status,
                updated_at,
            } => {
                let merchant_uuid = parse_merchant_uuid(&merchant_id)?;
                with_shared_txn(&self.db, |txn| {
                    Box::pin(async move {
                        let request_uuid = parse_uuid(&request_uuid, "request_uuid")?;
                        let Some(model) = service_requests::Entity::find()
                            .filter(service_requests::Column::MerchantId.eq(merchant_uuid))
                            .filter(service_requests::Column::Uuid.eq(request_uuid))
                            .one(txn)
                            .await
                            .map_err(|e| {
                                format!("query service request projection error: {}", e)
                            })?
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
                        active.update(txn).await.map_err(|e| {
                            format!("update service request projection error: {}", e)
                        })?;
                        Ok(())
                    })
                })
                .await?;
            }
        }

        Ok(())
    }
}

pub async fn spawn_service_request_listener(
    read_model_db: DatabaseConnection,
) -> Result<(), String> {
    let listener_event_store = event_store().await?;

    tokio::spawn(async move {
        let projection = match ServiceRequestProjection::new(read_model_db).await {
            Ok(projection) => projection,
            Err(err) => {
                eprintln!("failed to create service request projection: {}", err);
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
                        super::projection_listener_retry("service request", err, attempts)
                    }),
            )
            .start()
            .await
        {
            eprintln!(
                "service request projection listener exited with error: {}",
                err
            );
        }
    });

    Ok(())
}

fn parse_uuid(value: &str, field: &str) -> Result<Uuid, String> {
    Uuid::parse_str(value).map_err(|e| format!("invalid {} uuid {}: {}", field, value, e))
}

fn parse_optional_uuid(value: Option<&str>, field: &str) -> Result<Option<Uuid>, String> {
    value.map(|raw| parse_uuid(raw, field)).transpose()
}
