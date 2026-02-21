use crate::domain::models::user::User;
use crate::domain::queries::user::UserQuery;
use shared::user::{UserListQuery, PagedResult};
use crate::infrastructure::entity::users::{Column, Entity};
use crate::infrastructure::mappers::user_mapper::UserMapper;
use crate::infrastructure::tenant::with_tenant_txn;
use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, QuerySelect, PaginatorTrait, Condition, QueryOrder};
use sea_orm::prelude::*;

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
                println!("收到查询参数: {:?}", query);
                
                let mut select = Entity::find();
                let mut condition = Condition::all();

                // 添加筛选条件
                if let Some(name) = &query.name {
                    if !name.is_empty() {
                        condition = condition.add(Column::UserName.contains(name));
                        println!("添加姓名筛选: {}", name);
                    }
                }

                if let Some(status) = &query.status {
                    if !status.is_empty() {
                        condition = condition.add(Column::Status.eq(status));
                        println!("添加状态筛选: {}", status);
                    }
                }

                // 角色筛选优先使用 role 字段，保持对旧 is_admin 的兼容
                if let Some(role) = &query.role {
                    if !role.is_empty() {
                        match role.as_str() {
                            "admin" => {
                                let mut role_condition = Condition::any();
                                role_condition = role_condition.add(Column::Role.eq("admin"));
                                role_condition = role_condition.add(Column::IsAdmin.eq(Some(true)));
                                condition = condition.add(role_condition);
                                println!("添加管理员角色筛选");
                            }
                            "user" => {
                                condition = condition.add(Column::Role.ne("admin"));
                                println!("添加普通用户角色筛选");
                            }
                            _ => {
                                condition = condition.add(Column::Role.eq(role));
                            }
                        }
                    }
                }

                select = select.filter(condition);

                // 获取总数
                let total = select
                    .clone()
                    .count(txn)
                    .await
                    .map_err(|e| format!("Database count error: {}", e))?;

                // 分页查询，添加默认排序（按创建时间降序）
                let models = select
                    .order_by_desc(Column::InsertedAt)
                    .offset(Some((query.page - 1) * query.page_size))
                    .limit(Some(query.page_size))
                    .all(txn)
                    .await
                    .map_err(|e| format!("Database query error: {}", e))?;

                // 转换为domain对象
                let users: Vec<User> = models
                    .into_iter()
                    .map(|model| {
                        println!("数据库用户记录: uuid={}, name={}, status={}, is_admin={:?}", 
                            model.uuid, model.user_name, model.status, model.is_admin);
                        UserMapper::to_domain(model)
                    })
                    .collect();

                println!("查询结果: 总数={}, 返回用户数={}", total, users.len());

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
