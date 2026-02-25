use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use sea_orm::{
    ActiveModelTrait, DatabaseConnection, EntityTrait, IntoActiveModel, JoinType, QueryFilter,
    QuerySelect, Set,
};

use crate::domain::models::schedule::{ScheduleAssignment, ScheduleStatus};
use crate::domain::repositories::schedule::ScheduleRepository;
use crate::infrastructure::entity::orders::Column as OrderColumn;
use crate::infrastructure::entity::schedules::{
    ActiveModel as ScheduleActiveModel, Column as ScheduleColumn, Entity as ScheduleEntity,
    Model as ScheduleModel,
};
use crate::infrastructure::tenant::with_tenant_txn;

pub struct SeaOrmScheduleRepository {
    db: DatabaseConnection,
    schema_name: String,
}

impl SeaOrmScheduleRepository {
    pub fn new(db: DatabaseConnection, schema_name: String) -> Self {
        Self { db, schema_name }
    }

    fn to_domain(model: ScheduleModel) -> ScheduleAssignment {
        let status = ScheduleStatus::parse(&model.status).unwrap_or(ScheduleStatus::Planned);
        ScheduleAssignment {
            uuid: model.uuid.to_string(),
            order_id: model.order_id.to_string(),
            assigned_user_uuid: model.assigned_user_uuid.to_string(),
            start_at: model.start_at,
            end_at: model.end_at,
            status,
            notes: model.notes,
            inserted_at: model.inserted_at,
            updated_at: model.updated_at,
        }
    }
}

#[async_trait]
impl ScheduleRepository for SeaOrmScheduleRepository {
    fn find_by_order(
        &self,
        order_id: String,
    ) -> impl std::future::Future<Output = Result<Option<ScheduleAssignment>, String>> + Send {
        let db = self.db.clone();
        let schema_name = self.schema_name.clone();
        async move {
            with_tenant_txn(&db, &schema_name, |txn| {
                let order_id = order_id.clone();
                Box::pin(async move {
                    let order_uuid = Uuid::parse_str(&order_id)
                        .map_err(|e| format!("invalid order uuid: {}", e))?;
                    let model = ScheduleEntity::find()
                        .filter(ScheduleColumn::OrderId.eq(order_uuid))
                        .one(txn)
                        .await
                        .map_err(|e| format!("query schedule error: {}", e))?;
                    Ok(model.map(Self::to_domain))
                })
            })
            .await
        }
    }

    fn create_assignment(
        &self,
        assignment: ScheduleAssignment,
    ) -> impl std::future::Future<Output = Result<ScheduleAssignment, String>> + Send {
        let db = self.db.clone();
        let schema_name = self.schema_name.clone();
        async move {
            with_tenant_txn(&db, &schema_name, |txn| {
                Box::pin(async move {
                    let order_id = Uuid::parse_str(&assignment.order_id)
                        .map_err(|e| format!("invalid order uuid: {}", e))?;
                    let assigned_user_uuid = Uuid::parse_str(&assignment.assigned_user_uuid)
                        .map_err(|e| format!("invalid assigned user uuid: {}", e))?;
                    let active = ScheduleActiveModel {
                        uuid: Set(Uuid::new_v4()),
                        order_id: Set(order_id),
                        assigned_user_uuid: Set(assigned_user_uuid),
                        start_at: Set(assignment.start_at),
                        end_at: Set(assignment.end_at),
                        status: Set(assignment.status.as_str().to_string()),
                        notes: Set(assignment.notes),
                        ..Default::default()
                    };
                    let inserted = active
                        .insert(txn)
                        .await
                        .map_err(|e| format!("create schedule error: {}", e))?;
                    Ok(Self::to_domain(inserted))
                })
            })
            .await
        }
    }

    fn update_assignment(
        &self,
        order_id: String,
        assigned_user_uuid: String,
        start_at: DateTime<Utc>,
        end_at: DateTime<Utc>,
        notes: Option<String>,
    ) -> impl std::future::Future<Output = Result<ScheduleAssignment, String>> + Send {
        let db = self.db.clone();
        let schema_name = self.schema_name.clone();
        async move {
            with_tenant_txn(&db, &schema_name, |txn| {
                let order_id = order_id.clone();
                let assigned_user_uuid = assigned_user_uuid.clone();
                Box::pin(async move {
                    let order_uuid = Uuid::parse_str(&order_id)
                        .map_err(|e| format!("invalid order uuid: {}", e))?;
                    let assigned_user_uuid = Uuid::parse_str(&assigned_user_uuid)
                        .map_err(|e| format!("invalid assigned user uuid: {}", e))?;
                    let model = ScheduleEntity::find()
                        .filter(ScheduleColumn::OrderId.eq(order_uuid))
                        .one(txn)
                        .await
                        .map_err(|e| format!("query schedule error: {}", e))?
                        .ok_or_else(|| format!("schedule for order {} not found", order_id))?;

                    let mut active = model.into_active_model();
                    active.assigned_user_uuid = Set(assigned_user_uuid);
                    active.start_at = Set(start_at);
                    active.end_at = Set(end_at);
                    active.notes = Set(notes);
                    active.updated_at = Set(Utc::now());
                    let updated = active
                        .update(txn)
                        .await
                        .map_err(|e| format!("update schedule error: {}", e))?;
                    Ok(Self::to_domain(updated))
                })
            })
            .await
        }
    }

    fn delete_by_order(
        &self,
        order_id: String,
    ) -> impl std::future::Future<Output = Result<(), String>> + Send {
        let db = self.db.clone();
        let schema_name = self.schema_name.clone();
        async move {
            with_tenant_txn(&db, &schema_name, |txn| {
                let order_id = order_id.clone();
                Box::pin(async move {
                    let order_uuid = Uuid::parse_str(&order_id)
                        .map_err(|e| format!("invalid order uuid: {}", e))?;
                    if let Some(model) = ScheduleEntity::find()
                        .filter(ScheduleColumn::OrderId.eq(order_uuid))
                        .one(txn)
                        .await
                        .map_err(|e| format!("query schedule error: {}", e))?
                    {
                        model
                            .delete(txn)
                            .await
                            .map_err(|e| format!("delete schedule error: {}", e))?;
                    }
                    Ok(())
                })
            })
            .await
        }
    }

    fn update_status(
        &self,
        order_id: String,
        status: ScheduleStatus,
    ) -> impl std::future::Future<Output = Result<Option<ScheduleAssignment>, String>> + Send {
        let db = self.db.clone();
        let schema_name = self.schema_name.clone();
        async move {
            with_tenant_txn(&db, &schema_name, |txn| {
                let order_id = order_id.clone();
                Box::pin(async move {
                    let order_uuid = Uuid::parse_str(&order_id)
                        .map_err(|e| format!("invalid order uuid: {}", e))?;
                    let Some(model) = ScheduleEntity::find()
                        .filter(ScheduleColumn::OrderId.eq(order_uuid))
                        .one(txn)
                        .await
                        .map_err(|e| format!("query schedule error: {}", e))?
                    else {
                        return Ok(None);
                    };

                    let mut active = model.into_active_model();
                    active.status = Set(status.as_str().to_string());
                    active.updated_at = Set(Utc::now());
                    let updated = active
                        .update(txn)
                        .await
                        .map_err(|e| format!("update schedule status error: {}", e))?;
                    Ok(Some(Self::to_domain(updated)))
                })
            })
            .await
        }
    }

    fn find_conflict(
        &self,
        assigned_user_uuid: String,
        start_at: DateTime<Utc>,
        end_at: DateTime<Utc>,
        exclude_order_id: Option<String>,
    ) -> impl std::future::Future<Output = Result<Option<ScheduleAssignment>, String>> + Send {
        let db = self.db.clone();
        let schema_name = self.schema_name.clone();
        async move {
            with_tenant_txn(&db, &schema_name, |txn| {
                let assigned_user_uuid = assigned_user_uuid.clone();
                Box::pin(async move {
                    let assigned_user_uuid = Uuid::parse_str(&assigned_user_uuid)
                        .map_err(|e| format!("invalid assigned user uuid: {}", e))?;
                    let mut select = ScheduleEntity::find()
                        .join(
                            JoinType::InnerJoin,
                            crate::infrastructure::entity::schedules::Relation::Orders.def(),
                        )
                        .filter(ScheduleColumn::AssignedUserUuid.eq(assigned_user_uuid))
                        .filter(ScheduleColumn::StartAt.lt(end_at))
                        .filter(ScheduleColumn::EndAt.gt(start_at));

                    if let Some(order_id) = exclude_order_id {
                        let order_uuid = Uuid::parse_str(&order_id)
                            .map_err(|e| format!("invalid order uuid: {}", e))?;
                        select = select.filter(OrderColumn::Uuid.ne(order_uuid));
                    }

                    let active_statuses = vec![
                        "pending".to_string(),
                        "confirmed".to_string(),
                        "dispatching".to_string(),
                        "in_service".to_string(),
                    ];
                    select = select.filter(OrderColumn::Status.is_in(active_statuses));

                    let model = select
                        .one(txn)
                        .await
                        .map_err(|e| format!("query schedule conflict error: {}", e))?;

                    Ok(model.map(Self::to_domain))
                })
            })
            .await
        }
    }
}
