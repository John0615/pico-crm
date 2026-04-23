use chrono::{DateTime, NaiveDate, NaiveDateTime, TimeZone, Utc};
use sea_orm::entity::prelude::*;
use sea_orm::{
    ColumnTrait, Condition, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder, QuerySelect,
};

use crate::domain::crm::schedule::{ScheduleQuery as DomainScheduleQuery, ScheduleStatus};
use crate::infrastructure::entity::orders::{Column as OrderColumn, Entity as OrderEntity};
use crate::infrastructure::entity::schedules::{
    Column as ScheduleColumn, Entity as ScheduleEntity,
};
use crate::infrastructure::mappers::crm::schedule_mapper::ScheduleMapper;
use crate::infrastructure::tenant::{parse_merchant_uuid, with_shared_txn};
use shared::schedule::Schedule as SharedSchedule;
use shared::schedule::ScheduleQuery;

pub struct SeaOrmScheduleQuery {
    db: DatabaseConnection,
    merchant_id: String,
}

impl SeaOrmScheduleQuery {
    pub fn new(db: DatabaseConnection, merchant_id: String) -> Self {
        Self { db, merchant_id }
    }
}

impl DomainScheduleQuery for SeaOrmScheduleQuery {
    type Result = SharedSchedule;

    fn list_schedules(
        &self,
        query: ScheduleQuery,
    ) -> impl std::future::Future<Output = Result<(Vec<Self::Result>, u64), String>> + Send {
        let db = self.db.clone();
        let merchant_id = self.merchant_id.clone();
        async move {
            with_shared_txn(&db, |txn| {
                let merchant_id = merchant_id.clone();
                Box::pin(async move {
                    let merchant_uuid = parse_merchant_uuid(&merchant_id)?;
                    let mut select = OrderEntity::find().find_also_related(ScheduleEntity);
                    select = select.filter(build_schedule_condition(&query, merchant_uuid)?);

                    let total = select
                        .clone()
                        .count(txn)
                        .await
                        .map_err(|e| format!("query schedules count error: {}", e))?;

                    let models = select
                        .order_by_desc(ScheduleColumn::StartAt)
                        .order_by_desc(OrderColumn::InsertedAt)
                        .offset(Some((query.page - 1) * query.page_size))
                        .limit(Some(query.page_size))
                        .all(txn)
                        .await
                        .map_err(|e| format!("query schedules error: {}", e))?;

                    let items = models
                        .into_iter()
                        .map(|(order, schedule)| ScheduleMapper::to_view(order, schedule))
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
        let merchant_id = self.merchant_id.clone();
        async move {
            with_shared_txn(&db, |txn| {
                let merchant_id = merchant_id.clone();
                Box::pin(async move {
                    let merchant_uuid = parse_merchant_uuid(&merchant_id)?;
                    let order_uuid = Uuid::parse_str(&uuid)
                        .map_err(|e| format!("invalid schedule uuid: {}", e))?;
                    let model = OrderEntity::find()
                        .filter(OrderColumn::MerchantId.eq(merchant_uuid))
                        .filter(OrderColumn::Uuid.eq(order_uuid))
                        .find_also_related(ScheduleEntity)
                        .one(txn)
                        .await
                        .map_err(|e| format!("query schedule error: {}", e))?;
                    Ok(model.map(|(order, schedule)| ScheduleMapper::to_view(order, schedule)))
                })
            })
            .await
        }
    }
}

fn build_schedule_condition(
    query: &ScheduleQuery,
    merchant_uuid: Uuid,
) -> Result<Condition, String> {
    let mut condition = Condition::all();
    condition = condition.add(OrderColumn::MerchantId.eq(merchant_uuid));
    condition = condition.add(ScheduleColumn::Uuid.is_not_null());

    if let Some(status) = query.status.clone() {
        if !status.is_empty() {
            let schedule_status = ScheduleStatus::parse(&status)?;
            let mut status_condition = Condition::any();
            for value in schedule_status.order_statuses() {
                status_condition = status_condition.add(OrderColumn::Status.eq(*value));
            }
            condition = condition.add(status_condition);
        }
    }

    if let Some(user_uuid) = query.assigned_user_uuid.clone() {
        if !user_uuid.is_empty() {
            let user_uuid =
                Uuid::parse_str(&user_uuid).map_err(|e| format!("invalid user uuid: {}", e))?;
            condition = condition.add(ScheduleColumn::AssignedUserUuid.eq(user_uuid));
        }
    }

    if let Some(start) = query.start_date.as_deref().and_then(parse_datetime) {
        condition = condition.add(OrderColumn::ScheduledStartAt.gte(start));
    }
    if let Some(end) = query.end_date.as_deref().and_then(parse_datetime) {
        condition = condition.add(OrderColumn::ScheduledStartAt.lte(end));
    }

    Ok(condition)
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

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::{DbBackend, QueryTrait};

    #[test]
    fn generated_sql_contains_merchant_scope_for_schedule_list() {
        let merchant_uuid =
            Uuid::parse_str("11111111-1111-1111-1111-111111111111").expect("valid uuid");
        let query = ScheduleQuery {
            page: 1,
            page_size: 20,
            status: Some("planned".to_string()),
            assigned_user_uuid: None,
            start_date: None,
            end_date: None,
        };

        let sql = OrderEntity::find()
            .find_also_related(ScheduleEntity)
            .filter(build_schedule_condition(&query, merchant_uuid).expect("condition"))
            .build(DbBackend::Postgres)
            .to_string();

        assert!(
            sql.contains(r#""orders"."merchant_id" = '11111111-1111-1111-1111-111111111111'"#),
            "sql: {sql}"
        );
    }
}
