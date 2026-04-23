use crate::domain::identity::user::{User, UserQuery};
use crate::infrastructure::entity::after_sales_cases::{
    Column as AfterSalesColumn, Entity as AfterSalesEntity,
};
use crate::infrastructure::entity::after_sales_reworks::{
    Column as AfterSalesReworkColumn, Entity as AfterSalesReworkEntity,
};
use crate::infrastructure::entity::order_feedback::{
    Column as OrderFeedbackColumn, Entity as OrderFeedbackEntity,
};
use crate::infrastructure::entity::schedules::{
    Column as ScheduleColumn, Entity as ScheduleEntity,
};
use crate::infrastructure::entity::users::{Column, Entity};
use crate::infrastructure::mappers::identity::user_mapper::UserMapper;
use crate::infrastructure::tenant::with_shared_txn;
use sea_orm::prelude::*;
use sea_orm::sea_query::Expr;
use sea_orm::{
    Condition, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder,
    QuerySelect, Select, TransactionTrait,
};
use shared::user::{PagedResult, UserListQuery};
use std::collections::HashMap;

pub struct SeaOrmUserQuery {
    db: DatabaseConnection,
    merchant_id: String,
}

impl SeaOrmUserQuery {
    pub fn new(db: DatabaseConnection, merchant_id: String) -> Self {
        Self { db, merchant_id }
    }
}

impl UserQuery for SeaOrmUserQuery {
    type Result = User;

    async fn get_user(&self, user_name: &str) -> Result<Option<Self::Result>, String> {
        let db = self.db.clone();
        let merchant_id = self.merchant_id.clone();
        let user_name = user_name.to_string();
        with_shared_txn(&db, |txn| {
            let merchant_id = merchant_id.clone();
            let user_name = user_name.clone();
            Box::pin(async move {
                let merchant_uuid = parse_merchant_uuid(&merchant_id)?;
                let user_model = Entity::find()
                    .filter(Column::MerchantUuid.eq(merchant_uuid))
                    .filter(Column::UserName.eq(user_name))
                    .one(txn)
                    .await
                    .map_err(|e| format!("Database error: {}", e))?;

                match user_model {
                    Some(model) => Ok(Some(UserMapper::to_domain(model))),
                    None => Ok(None),
                }
            })
        })
        .await
    }

    async fn list_users(&self, query: UserListQuery) -> Result<PagedResult<Self::Result>, String> {
        let db = self.db.clone();
        let merchant_id = self.merchant_id.clone();
        with_shared_txn(&db, |txn| {
            let merchant_id = merchant_id.clone();
            Box::pin(async move {
                let merchant_uuid = parse_merchant_uuid(&merchant_id)?;
                let select = apply_user_filters(
                    Entity::find().filter(Column::MerchantUuid.eq(merchant_uuid)),
                    &query,
                );

                let total = select
                    .clone()
                    .count(txn)
                    .await
                    .map_err(|e| format!("Database count error: {}", e))?;

                let models = select
                    .order_by_desc(Column::InsertedAt)
                    .offset(Some((query.page - 1) * query.page_size))
                    .limit(Some(query.page_size))
                    .all(txn)
                    .await
                    .map_err(|e| format!("Database query error: {}", e))?;

                let user_ids = models.iter().map(|model| model.uuid).collect::<Vec<_>>();
                let performance = load_user_performance(txn, &user_ids).await?;

                let users: Vec<User> = models
                    .into_iter()
                    .map(|model| {
                        let mut user = UserMapper::to_domain(model.clone());
                        if let Some(stats) = performance.get(&model.uuid) {
                            user.completed_service_count = Some(stats.completed_service_count);
                            user.feedback_count = Some(stats.feedback_count);
                            user.average_rating = stats.average_rating;
                            user.after_sales_case_count = Some(stats.after_sales_case_count);
                            user.complaint_case_count = Some(stats.complaint_case_count);
                            user.refund_case_count = Some(stats.refund_case_count);
                            user.rework_count = Some(stats.rework_count);
                        }
                        user
                    })
                    .collect();

                Ok(PagedResult {
                    items: users,
                    total,
                })
            })
        })
        .await
    }

    async fn find_user_by_uuid(&self, uuid: &str) -> Result<Option<Self::Result>, String> {
        let db = self.db.clone();
        let merchant_id = self.merchant_id.clone();
        let uuid = uuid.to_string();
        with_shared_txn(&db, |txn| {
            let merchant_id = merchant_id.clone();
            let uuid = uuid.clone();
            Box::pin(async move {
                let merchant_uuid = parse_merchant_uuid(&merchant_id)?;
                let uuid = Uuid::parse_str(&uuid).map_err(|e| format!("解析uuid失败: {}", e))?;
                let user = Entity::find()
                    .filter(Column::MerchantUuid.eq(merchant_uuid))
                    .filter(Column::Uuid.eq(uuid))
                    .one(txn)
                    .await
                    .map_err(|e| format!("查询用户失败: {}", e))?;

                let Some(model) = user else {
                    return Ok(None);
                };
                let performance = load_user_performance(txn, &[model.uuid]).await?;
                let mut user = UserMapper::to_domain(model.clone());
                if let Some(stats) = performance.get(&model.uuid) {
                    user.completed_service_count = Some(stats.completed_service_count);
                    user.feedback_count = Some(stats.feedback_count);
                    user.average_rating = stats.average_rating;
                    user.after_sales_case_count = Some(stats.after_sales_case_count);
                    user.complaint_case_count = Some(stats.complaint_case_count);
                    user.refund_case_count = Some(stats.refund_case_count);
                    user.rework_count = Some(stats.rework_count);
                }

                Ok(Some(user))
            })
        })
        .await
    }
}

fn parse_merchant_uuid(merchant_id: &str) -> Result<Uuid, String> {
    Uuid::parse_str(merchant_id).map_err(|e| format!("invalid merchant uuid: {}", e))
}

#[derive(Debug, Clone, Default)]
struct UserPerformanceStats {
    completed_service_count: u64,
    feedback_count: u64,
    average_rating: Option<f64>,
    after_sales_case_count: u64,
    complaint_case_count: u64,
    refund_case_count: u64,
    rework_count: u64,
}

async fn load_user_performance<C>(
    txn: &C,
    user_ids: &[Uuid],
) -> Result<HashMap<Uuid, UserPerformanceStats>, String>
where
    C: TransactionTrait + sea_orm::ConnectionTrait,
{
    if user_ids.is_empty() {
        return Ok(HashMap::new());
    }

    let schedule_items = ScheduleEntity::find()
        .filter(ScheduleColumn::AssignedUserUuid.is_in(user_ids.iter().copied()))
        .filter(ScheduleColumn::Status.eq("done"))
        .all(txn)
        .await
        .map_err(|e| format!("query user completed schedules error: {}", e))?;

    let feedback_items = OrderFeedbackEntity::find()
        .filter(OrderFeedbackColumn::UserUuid.is_in(user_ids.iter().copied()))
        .all(txn)
        .await
        .map_err(|e| format!("query user feedback stats error: {}", e))?;

    let schedule_order_map = schedule_items
        .iter()
        .map(|item| (item.order_uuid, item.assigned_user_uuid))
        .collect::<HashMap<_, _>>();
    let order_ids = schedule_order_map.keys().copied().collect::<Vec<_>>();
    let after_sales_cases = if order_ids.is_empty() {
        Vec::new()
    } else {
        AfterSalesEntity::find()
            .filter(AfterSalesColumn::OrderUuid.is_in(order_ids.clone()))
            .all(txn)
            .await
            .map_err(|e| format!("query user after sales cases error: {}", e))?
    };

    let mut stats_map = HashMap::<Uuid, UserPerformanceStats>::new();

    for item in schedule_items {
        let entry = stats_map.entry(item.assigned_user_uuid).or_default();
        entry.completed_service_count += 1;
    }

    let mut rating_sums = HashMap::<Uuid, (i64, u64)>::new();
    for item in feedback_items {
        let Some(user_uuid) = item.user_uuid else {
            continue;
        };
        let entry = stats_map.entry(user_uuid).or_default();
        entry.feedback_count += 1;

        if let Some(rating) = item.rating {
            let rating_entry = rating_sums.entry(user_uuid).or_insert((0, 0));
            rating_entry.0 += rating as i64;
            rating_entry.1 += 1;
        }
    }

    let mut case_user_map = HashMap::<Uuid, Uuid>::new();
    for case in after_sales_cases {
        let Some(user_uuid) = schedule_order_map.get(&case.order_uuid).copied() else {
            continue;
        };
        case_user_map.insert(case.uuid, user_uuid);
        let entry = stats_map.entry(user_uuid).or_default();
        entry.after_sales_case_count += 1;
        if case.case_type == "complaint" {
            entry.complaint_case_count += 1;
        }
        if case
            .refund_amount_cents
            .map(|value| value > 0)
            .unwrap_or(false)
            || case.case_type == "refund"
        {
            entry.refund_case_count += 1;
        }
    }

    if !case_user_map.is_empty() {
        let case_ids = case_user_map.keys().copied().collect::<Vec<_>>();
        let reworks = AfterSalesReworkEntity::find()
            .filter(AfterSalesReworkColumn::CaseUuid.is_in(case_ids))
            .all(txn)
            .await
            .map_err(|e| format!("query user after sales reworks error: {}", e))?;
        for rework in reworks {
            if let Some(user_uuid) = case_user_map.get(&rework.case_uuid).copied() {
                let entry = stats_map.entry(user_uuid).or_default();
                entry.rework_count += 1;
            }
        }
    }

    for (user_uuid, (sum, count)) in rating_sums {
        if let Some(entry) = stats_map.get_mut(&user_uuid) {
            if count > 0 {
                entry.average_rating = Some((sum as f64) / (count as f64));
            }
        }
    }

    Ok(stats_map)
}

fn apply_user_filters(select: Select<Entity>, query: &UserListQuery) -> Select<Entity> {
    let mut condition = Condition::all();

    if let Some(name) = &query.name {
        if !name.is_empty() {
            condition = condition.add(Column::UserName.contains(name));
        }
    }

    if let Some(status) = &query.status {
        if !status.is_empty() {
            condition = condition.add(Column::Status.eq(status));
        }
    }

    if query.dispatchable_only.unwrap_or(false) {
        condition = condition.add(Column::Role.eq("user"));
        condition = condition.add(Column::Status.eq("active"));
        condition = condition.add(Column::EmploymentStatus.eq("active"));
        condition = condition.add(Column::HealthStatus.eq("healthy"));
    } else if let Some(role) = &query.role {
        if !role.is_empty() {
            match role.as_str() {
                "admin" => {
                    let mut role_condition = Condition::any();
                    role_condition = role_condition.add(Column::Role.eq("admin"));
                    role_condition = role_condition.add(Column::IsAdmin.eq(Some(true)));
                    condition = condition.add(role_condition);
                }
                _ => {
                    condition = condition.add(Column::Role.eq(role));
                }
            }
        }
    }

    if let Some(employment_status) = &query.employment_status {
        if !employment_status.is_empty() {
            condition = condition.add(Column::EmploymentStatus.eq(employment_status));
        }
    } else if query.role.as_deref() == Some("user") && !query.dispatchable_only.unwrap_or(false) {
        condition = condition.add(Column::EmploymentStatus.ne("resigned"));
    }

    if let Some(skill) = &query.skill {
        if !skill.is_empty() {
            let pattern = format!("%\"{}\"%", skill.trim());
            condition = condition.add(Expr::col(Column::Skills).cast_as("text").like(pattern));
        }
    }

    select.filter(condition)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::{DbBackend, QueryTrait};

    #[test]
    fn merchant_scope_uses_merchant_id_column() {
        let merchant_uuid = Uuid::parse_str("11111111-1111-1111-1111-111111111111").expect("uuid");
        let sql = Entity::find()
            .filter(Column::MerchantUuid.eq(merchant_uuid))
            .build(DbBackend::Postgres)
            .to_string();

        assert!(
            sql.contains(r#""users"."merchant_id" = '11111111-1111-1111-1111-111111111111'"#),
            "sql: {sql}"
        );
        assert!(!sql.contains(r#""users"."merchant_uuid""#), "sql: {sql}");
    }

    #[test]
    fn dispatchable_filter_requires_active_employee_user() {
        let sql = apply_user_filters(
            Entity::find(),
            &UserListQuery {
                page: 1,
                page_size: 20,
                name: None,
                role: None,
                status: None,
                employment_status: None,
                skill: None,
                dispatchable_only: Some(true),
            },
        )
        .build(DbBackend::Postgres)
        .to_string();

        assert!(sql.contains(r#""users"."role" = 'user'"#), "sql: {sql}");
        assert!(sql.contains(r#""users"."status" = 'active'"#), "sql: {sql}");
        assert!(
            sql.contains(r#""users"."employment_status" = 'active'"#),
            "sql: {sql}"
        );
        assert!(
            sql.contains(r#""users"."health_status" = 'healthy'"#),
            "sql: {sql}"
        );
    }

    #[test]
    fn skill_filter_matches_json_array_text() {
        let sql = apply_user_filters(
            Entity::find(),
            &UserListQuery {
                page: 1,
                page_size: 20,
                name: None,
                role: Some("user".to_string()),
                status: None,
                employment_status: None,
                skill: Some("保洁".to_string()),
                dispatchable_only: None,
            },
        )
        .build(DbBackend::Postgres)
        .to_string();

        assert!(sql.contains(r#"CAST("skills" AS text) LIKE"#), "sql: {sql}");
        assert!(sql.contains("保洁"), "sql: {sql}");
        assert!(
            sql.contains(r#""users"."employment_status" <> 'resigned'"#),
            "sql: {sql}"
        );
    }
}
