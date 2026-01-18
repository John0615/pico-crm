use crate::domain::models::user::User;
use crate::domain::queries::user::UserQuery;
use shared::user::{UserListQuery, PagedResult};
use crate::infrastructure::entity::users::{Column, Entity};
use crate::infrastructure::mappers::user_mapper::UserMapper;
use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, QuerySelect, PaginatorTrait, Condition};
use sea_orm::prelude::*;

pub struct SeaOrmUserQuery {
    db: DatabaseConnection,
}

impl SeaOrmUserQuery {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

impl UserQuery for SeaOrmUserQuery {
    type Result = User;

    async fn get_user(&self, user_name: &str) -> Result<Option<Self::Result>, String> {
        let user_model = Entity::find()
            .filter(Column::UserName.eq(user_name))
            .one(&self.db)
            .await
            .map_err(|e| format!("Database error: {}", e))?;

        match user_model {
            Some(model) => Ok(Some(UserMapper::to_domain(model))),
            None => Ok(None),
        }
    }

    async fn list_users(&self, query: UserListQuery) -> Result<PagedResult<Self::Result>, String> {
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

        // 如果有角色筛选，需要根据is_admin字段进行转换
        if let Some(role) = &query.role {
            if !role.is_empty() {
                match role.as_str() {
                    "管理员" => {
                        condition = condition.add(Column::IsAdmin.eq(Some(true)));
                        println!("添加管理员角色筛选");
                    },
                    "普通用户" => {
                        condition = condition.add(Column::IsAdmin.eq(Some(false)));
                        println!("添加普通用户角色筛选");
                    },
                    _ => {} // 其他角色暂时忽略
                }
            }
        }

        select = select.filter(condition);

        // 获取总数
        let total = select
            .clone()
            .count(&self.db)
            .await
            .map_err(|e| format!("Database count error: {}", e))?;

        // 分页查询
        let models = select
            .offset(Some((query.page - 1) * query.page_size))
            .limit(Some(query.page_size))
            .all(&self.db)
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
    }
}