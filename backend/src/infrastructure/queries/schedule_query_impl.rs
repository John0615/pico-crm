use chrono::{DateTime, NaiveDate, NaiveDateTime, TimeZone, Utc};
use sea_orm::{
    ColumnTrait, Condition, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder, QuerySelect,
};
use sea_orm::entity::prelude::*;

use crate::domain::models::schedule::ScheduleStatus;
use crate::domain::queries::schedule::ScheduleQuery as DomainScheduleQuery;
use crate::infrastructure::entity::orders::{Column, Entity};
use crate::infrastructure::mappers::schedule_mapper::ScheduleMapper;
use crate::infrastructure::tenant::with_tenant_txn;
use shared::schedule::Schedule as SharedSchedule;
use shared::schedule::ScheduleQuery;

pub struct SeaOrmScheduleQuery {
    db: DatabaseConnection,
    schema_name: String,
}

impl SeaOrmScheduleQuery {
    pub fn new(db: DatabaseConnection, schema_name: String) -> Self {
        Self { db, schema_name }
    }
}

impl DomainScheduleQuery for SeaOrmScheduleQuery {
    type Result = SharedSchedule;

    fn list_schedules(
        &self,
        query: ScheduleQuery,
    ) -> impl std::future::Future<Output = Result<(Vec<Self::Result>, u64), String>> + Send {
        let db = self.db.clone();
        let schema_name = self.schema_name.clone();
        async move {
            with_tenant_txn(&db, &schema_name, |txn| {
                Box::pin(async move {
                    let mut select = Entity::find();
                    let mut condition = Condition::all();

                    if let Some(status) = query.status {
                        if !status.is_empty() {
                            let schedule_status = ScheduleStatus::parse(&status)?;
                            let mut status_condition = Condition::any();
                            for value in schedule_status.order_statuses() {
                                status_condition = status_condition.add(Column::Status.eq(*value));
                            }
                            condition = condition.add(status_condition);
                        }
                    }

                    if let Some(user_uuid) = query.assigned_user_uuid {
                        if !user_uuid.is_empty() {
                            let user_uuid = Uuid::parse_str(&user_uuid)
                                .map_err(|e| format!("invalid user uuid: {}", e))?;
                            condition = condition.add(Column::AssignedUserUuid.eq(user_uuid));
                        }
                    }

                    if let Some(start) = query.start_date.as_deref().and_then(parse_datetime) {
                        condition = condition.add(Column::ScheduledStartAt.gte(start));
                    }
                    if let Some(end) = query.end_date.as_deref().and_then(parse_datetime) {
                        condition = condition.add(Column::ScheduledStartAt.lte(end));
                    }

                    select = select.filter(condition);

                    let total = select
                        .clone()
                        .count(txn)
                        .await
                        .map_err(|e| format!("query schedules count error: {}", e))?;

                    let models = select
                        .order_by_desc(Column::ScheduledStartAt)
                        .order_by_desc(Column::InsertedAt)
                        .offset(Some((query.page - 1) * query.page_size))
                        .limit(Some(query.page_size))
                        .all(txn)
                        .await
                        .map_err(|e| format!("query schedules error: {}", e))?;

                    let items = models
                        .into_iter()
                        .map(ScheduleMapper::to_view)
                        .collect();

                    Ok((items, total))
                })
            })
            .await
        }
    }

    fn get_schedule(
        &self,
        uuid: String,
    ) -> impl std::future::Future<Output = Result<Option<Self::Result>, String>> + Send {
        let db = self.db.clone();
        let schema_name = self.schema_name.clone();
        async move {
            with_tenant_txn(&db, &schema_name, |txn| {
                Box::pin(async move {
                    let order_uuid = Uuid::parse_str(&uuid)
                        .map_err(|e| format!("invalid schedule uuid: {}", e))?;
                    let model = Entity::find_by_id(order_uuid)
                        .one(txn)
                        .await
                        .map_err(|e| format!("query schedule error: {}", e))?;
                    Ok(model.map(ScheduleMapper::to_view))
                })
            })
            .await
        }
    }
}

fn parse_datetime(value: &str) -> Option<DateTime<Utc>> {
    if value.trim().is_empty() {
        return None;
    }
    if let Ok(dt) = DateTime::parse_from_rfc3339(value) {
        return Some(dt.with_timezone(&Utc));
    }
    if let Ok(dt) = NaiveDateTime::parse_from_str(value, "%Y-%m-%d %H:%M:%S") {
        return Some(Utc.from_utc_datetime(&dt));
    }
    if let Ok(date) = NaiveDate::parse_from_str(value, "%Y-%m-%d") {
        if let Some(dt) = date.and_hms_opt(0, 0, 0) {
            return Some(Utc.from_utc_datetime(&dt));
        }
    }
    None
}
