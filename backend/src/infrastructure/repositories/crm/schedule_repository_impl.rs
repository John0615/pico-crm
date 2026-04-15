use async_trait::async_trait;
use chrono::{DateTime, Utc};
use disintegrate::{EventSourcedStateStore, LoadState, NoSnapshot};
use sea_orm::entity::prelude::*;
use sea_orm::{DatabaseConnection, EntityTrait, JoinType, QueryFilter, QuerySelect};

use crate::domain::crm::schedule::{
    CreateScheduleAssignmentDecision, DeleteScheduleDecision, ScheduleAssignment,
    ScheduleRepository, ScheduleState, ScheduleStatus, UpdateScheduleAssignmentDecision,
    UpdateScheduleStatusDecision,
};
use crate::infrastructure::entity::orders::Column as OrderColumn;
use crate::infrastructure::entity::schedules::{
    Column as ScheduleColumn, Entity as ScheduleEntity, Model as ScheduleModel,
};
use crate::infrastructure::event_store::schedule::event_store;
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

    async fn load_schedule_state(
        schema_name: &str,
        order_id: &str,
    ) -> Result<ScheduleState, String> {
        let event_store = event_store().await?;
        let state_store = EventSourcedStateStore::new(event_store, NoSnapshot);
        let loaded_state = state_store
            .load(ScheduleState::new(
                schema_name.to_string(),
                order_id.to_string(),
            ))
            .await
            .map_err(|e| format!("load schedule state error: {}", e))?;
        Ok(loaded_state.state().clone())
    }

    async fn load_schedule_from_events(
        schema_name: &str,
        order_id: &str,
    ) -> Result<Option<ScheduleAssignment>, String> {
        let state = Self::load_schedule_state(schema_name, order_id).await?;
        if !state.exists {
            return Ok(None);
        }
        Ok(Some(state.to_domain()?))
    }
}

#[async_trait]
impl ScheduleRepository for SeaOrmScheduleRepository {
    fn find_by_order(
        &self,
        order_id: String,
    ) -> impl std::future::Future<Output = Result<Option<ScheduleAssignment>, String>> + Send {
        let schema_name = self.schema_name.clone();
        async move { SeaOrmScheduleRepository::load_schedule_from_events(&schema_name, &order_id).await }
    }

    fn create_assignment(
        &self,
        assignment: ScheduleAssignment,
    ) -> impl std::future::Future<Output = Result<ScheduleAssignment, String>> + Send {
        let schema_name = self.schema_name.clone();
        async move {
            let mut assignment = assignment;
            if assignment.uuid.trim().is_empty() {
                assignment.uuid = Uuid::new_v4().to_string();
            }
            let order_id = assignment.order_id.clone();
            let event_store = event_store().await?;
            let decision_maker = disintegrate_postgres::decision_maker(event_store, NoSnapshot);
            decision_maker
                .make(CreateScheduleAssignmentDecision::new(
                    schema_name.clone(),
                    assignment,
                ))
                .await
                .map_err(|e| format!("create schedule assignment decision error: {}", e))?;

            SeaOrmScheduleRepository::load_schedule_from_events(&schema_name, &order_id)
                .await?
                .ok_or_else(|| format!("schedule for order {} not found after creation", order_id))
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
        let schema_name = self.schema_name.clone();
        async move {
            SeaOrmScheduleRepository::load_schedule_from_events(&schema_name, &order_id)
                .await?
                .ok_or_else(|| format!("schedule for order {} not found", order_id.clone()))?;

            let event_store = event_store().await?;
            let decision_maker = disintegrate_postgres::decision_maker(event_store, NoSnapshot);
            decision_maker
                .make(UpdateScheduleAssignmentDecision::new(
                    schema_name.clone(),
                    order_id.clone(),
                    assigned_user_uuid,
                    start_at,
                    end_at,
                    notes,
                    Utc::now(),
                ))
                .await
                .map_err(|e| format!("update schedule assignment decision error: {}", e))?;

            SeaOrmScheduleRepository::load_schedule_from_events(&schema_name, &order_id)
                .await?
                .ok_or_else(|| {
                    format!(
                        "schedule for order {} not found after assignment update",
                        order_id
                    )
                })
        }
    }

    fn delete_by_order(
        &self,
        order_id: String,
    ) -> impl std::future::Future<Output = Result<(), String>> + Send {
        let schema_name = self.schema_name.clone();
        async move {
            if SeaOrmScheduleRepository::load_schedule_from_events(&schema_name, &order_id)
                .await?
                .is_none()
            {
                return Ok(());
            }

            let event_store = event_store().await?;
            let decision_maker = disintegrate_postgres::decision_maker(event_store, NoSnapshot);
            decision_maker
                .make(DeleteScheduleDecision::new(
                    schema_name,
                    order_id,
                    Utc::now(),
                ))
                .await
                .map_err(|e| format!("delete schedule decision error: {}", e))?;
            Ok(())
        }
    }

    fn update_status(
        &self,
        order_id: String,
        status: ScheduleStatus,
    ) -> impl std::future::Future<Output = Result<Option<ScheduleAssignment>, String>> + Send {
        let schema_name = self.schema_name.clone();
        async move {
            if SeaOrmScheduleRepository::load_schedule_from_events(&schema_name, &order_id)
                .await?
                .is_none()
            {
                return Ok(None);
            }

            let event_store = event_store().await?;
            let decision_maker = disintegrate_postgres::decision_maker(event_store, NoSnapshot);
            decision_maker
                .make(UpdateScheduleStatusDecision::new(
                    schema_name.clone(),
                    order_id.clone(),
                    status,
                    Utc::now(),
                ))
                .await
                .map_err(|e| format!("update schedule status decision error: {}", e))?;

            SeaOrmScheduleRepository::load_schedule_from_events(&schema_name, &order_id).await
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
