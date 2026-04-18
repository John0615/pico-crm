use crate::domain::identity::user::{User, UserQuery};
use crate::infrastructure::entity::users::{Column, Entity};
use crate::infrastructure::mappers::identity::user_mapper::UserMapper;
use crate::infrastructure::tenant::with_tenant_txn;
use sea_orm::prelude::*;
use sea_orm::sea_query::Expr;
use sea_orm::{
    Condition, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder,
    QuerySelect, Select,
};
use shared::user::{PagedResult, UserListQuery};

pub struct SeaOrmUserQuery {
    db: DatabaseConnection,
    schema_name: String,
}

impl SeaOrmUserQuery {
    pub fn new(db: DatabaseConnection, schema_name: String) -> Self {
        Self { db, schema_name }
    }
}

impl UserQuery for SeaOrmUserQuery {
    type Result = User;

    async fn get_user(&self, user_name: &str) -> Result<Option<Self::Result>, String> {
        let db = self.db.clone();
        let schema_name = self.schema_name.clone();
        let user_name = user_name.to_string();
        with_tenant_txn(&db, &schema_name, |txn| {
            let user_name = user_name.clone();
            Box::pin(async move {
                let user_model = Entity::find()
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
        let schema_name = self.schema_name.clone();
        with_tenant_txn(&db, &schema_name, |txn| {
            Box::pin(async move {
                let select = apply_user_filters(Entity::find(), &query);

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

                let users: Vec<User> = models.into_iter().map(UserMapper::to_domain).collect();

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
        let schema_name = self.schema_name.clone();
        let uuid = uuid.to_string();
        with_tenant_txn(&db, &schema_name, |txn| {
            let uuid = uuid.clone();
            Box::pin(async move {
                let uuid = Uuid::parse_str(&uuid).map_err(|e| format!("解析uuid失败: {}", e))?;
                let user = Entity::find()
                    .filter(Column::Uuid.eq(uuid))
                    .one(txn)
                    .await
                    .map_err(|e| format!("查询用户失败: {}", e))?;

                Ok(user.map(UserMapper::to_domain))
            })
        })
        .await
    }
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
