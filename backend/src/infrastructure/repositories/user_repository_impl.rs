use sea_orm::entity::prelude::*;

use crate::{
    domain::{
        models::user::User,
        repositories::user::UserRepository,
    },
    infrastructure::entity::users::{Column, Entity},
    infrastructure::mappers::user_mapper::UserMapper,
};

use async_trait::async_trait;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait};

pub struct SeaOrmUserRepository {
    db: DatabaseConnection,
}

impl SeaOrmUserRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl UserRepository for SeaOrmUserRepository {
    fn create_user(
        &self,
        user: User,
    ) -> impl std::future::Future<Output = Result<User, String>> + Send {
        async move {
            // 转换为Entity
            let entity = UserMapper::to_active_entity(user);
            let new_entity = entity
                .insert(&self.db)
                .await
                .map_err(|e| format!("create user database error: {}", e))?;

            let new_user = UserMapper::to_domain(new_entity);
            Ok(new_user)
        }
    }

    fn update_user(
        &self,
        user: User,
    ) -> impl std::future::Future<Output = Result<User, String>> + Send {
        async move {
            // 根据 uuid 查询原始数据
            let uuid = Uuid::parse_str(&user.uuid).expect("解析uuid失败！");
            let original_user = Entity::find()
                .filter(Column::Uuid.eq(uuid))
                .one(&self.db)
                .await
                .map_err(|e| format!("查询原始数据失败: {}", e))?
                .ok_or_else(|| format!("未找到 uuid 为 {} 的用户", user.uuid))?;

            // 转换为 ActiveModel
            let active_user = UserMapper::to_update_active_entity(user, original_user);

            // 执行更新
            let updated = active_user
                .update(&self.db)
                .await
                .map_err(|e| format!("更新失败: {}", e))?;
            let updated = UserMapper::to_domain(updated);

            Ok(updated)
        }
    }

    fn delete_user(
        &self,
        uuid: String,
    ) -> impl std::future::Future<Output = Result<(), String>> + Send {
        async move {
            let uuid = Uuid::parse_str(&uuid).expect("解析uuid失败！");
            Entity::delete_by_id(uuid)
                .exec(&self.db)
                .await
                .map_err(|e| format!("删除失败: {}", e))
                .and_then(|res| {
                    if res.rows_affected > 0 {
                        Ok(())
                    } else {
                        Err("未找到记录".to_string())
                    }
                })
        }
    }

    // 命令操作相关的查询方法
    fn find_user_by_uuid(
        &self,
        uuid: &str,
    ) -> impl std::future::Future<Output = Result<Option<User>, String>> + Send {
        async move {
            let uuid = Uuid::parse_str(uuid).map_err(|e| format!("解析uuid失败: {}", e))?;
            let user = Entity::find()
                .filter(Column::Uuid.eq(uuid))
                .one(&self.db)
                .await
                .map_err(|e| format!("查询用户失败: {}", e))?;

            Ok(user.map(UserMapper::to_domain))
        }
    }

    fn find_user_by_username(
        &self,
        username: &str,
    ) -> impl std::future::Future<Output = Result<Option<User>, String>> + Send {
        async move {
            let user = Entity::find()
                .filter(Column::UserName.eq(username))
                .one(&self.db)
                .await
                .map_err(|e| format!("查询用户失败: {}", e))?;

            Ok(user.map(UserMapper::to_domain))
        }
    }

    fn find_user_by_email(
        &self,
        email: &str,
    ) -> impl std::future::Future<Output = Result<Option<User>, String>> + Send {
        async move {
            let user = Entity::find()
                .filter(Column::Email.eq(email))
                .one(&self.db)
                .await
                .map_err(|e| format!("查询用户失败: {}", e))?;

            Ok(user.map(UserMapper::to_domain))
        }
    }

    fn find_user_by_phone_number(
        &self,
        phone_number: &str,
    ) -> impl std::future::Future<Output = Result<Option<User>, String>> + Send {
        async move {
            let user = Entity::find()
                .filter(Column::PhoneNumber.eq(phone_number))
                .one(&self.db)
                .await
                .map_err(|e| format!("查询用户失败: {}", e))?;

            Ok(user.map(UserMapper::to_domain))
        }
    }
}
