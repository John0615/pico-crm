use sea_orm::entity::prelude::*;

use crate::{
    domain::identity::user::{User, UserRepository},
    infrastructure::entity::users::{Column, Entity},
    infrastructure::mappers::identity::user_mapper::UserMapper,
    infrastructure::tenant::with_shared_txn,
};

use async_trait::async_trait;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait};

pub struct SeaOrmUserRepository {
    db: DatabaseConnection,
    merchant_id: String,
}

impl SeaOrmUserRepository {
    pub fn new(db: DatabaseConnection, merchant_id: String) -> Self {
        Self { db, merchant_id }
    }
}

#[async_trait]
impl UserRepository for SeaOrmUserRepository {
    fn create_user(
        &self,
        user: User,
    ) -> impl std::future::Future<Output = Result<User, String>> + Send {
        let db = self.db.clone();
        let merchant_id = self.merchant_id.clone();
        async move {
            with_shared_txn(&db, |txn| {
                let merchant_id = merchant_id.clone();
                Box::pin(async move {
                    let mut user = user;
                    if user.merchant_uuid.is_none() {
                        user.merchant_uuid = Some(merchant_id.clone());
                    }
                    let merchant_uuid = parse_merchant_uuid(&merchant_id)?;
                    // 转换为Entity
                    let entity = UserMapper::to_active_entity(user);
                    let new_entity = entity
                        .insert(txn)
                        .await
                        .map_err(|e| format!("create user database error: {}", e))?;
                    if new_entity.merchant_uuid != Some(merchant_uuid) {
                        return Err("created user merchant mismatch".to_string());
                    }

                    let new_user = UserMapper::to_domain(new_entity);
                    Ok(new_user)
                })
            })
            .await
        }
    }

    fn update_user(
        &self,
        user: User,
    ) -> impl std::future::Future<Output = Result<User, String>> + Send {
        let db = self.db.clone();
        let merchant_id = self.merchant_id.clone();
        async move {
            with_shared_txn(&db, |txn| {
                let merchant_id = merchant_id.clone();
                Box::pin(async move {
                    let merchant_uuid = parse_merchant_uuid(&merchant_id)?;
                    // 根据 uuid 查询原始数据
                    let uuid = Uuid::parse_str(&user.uuid).expect("解析uuid失败！");
                    let original_user = Entity::find()
                        .filter(Column::MerchantUuid.eq(merchant_uuid))
                        .filter(Column::Uuid.eq(uuid))
                        .one(txn)
                        .await
                        .map_err(|e| format!("查询原始数据失败: {}", e))?
                        .ok_or_else(|| format!("未找到 uuid 为 {} 的用户", user.uuid))?;

                    // 转换为 ActiveModel
                    let mut user = user;
                    user.merchant_uuid = Some(merchant_id);
                    let active_user = UserMapper::to_update_active_entity(user, original_user);

                    // 执行更新
                    let updated = active_user
                        .update(txn)
                        .await
                        .map_err(|e| format!("更新失败: {}", e))?;
                    let updated = UserMapper::to_domain(updated);

                    Ok(updated)
                })
            })
            .await
        }
    }

    fn delete_user(
        &self,
        uuid: String,
    ) -> impl std::future::Future<Output = Result<(), String>> + Send {
        let db = self.db.clone();
        let merchant_id = self.merchant_id.clone();
        async move {
            with_shared_txn(&db, |txn| {
                let merchant_id = merchant_id.clone();
                Box::pin(async move {
                    let merchant_uuid = parse_merchant_uuid(&merchant_id)?;
                    let uuid = Uuid::parse_str(&uuid).expect("解析uuid失败！");
                    Entity::delete_many()
                        .filter(Column::MerchantUuid.eq(merchant_uuid))
                        .filter(Column::Uuid.eq(uuid))
                        .exec(txn)
                        .await
                        .map_err(|e| format!("删除失败: {}", e))
                        .and_then(|res| {
                            if res.rows_affected > 0 {
                                Ok(())
                            } else {
                                Err("未找到记录".to_string())
                            }
                        })
                })
            })
            .await
        }
    }

    // 命令操作相关的查询方法
    fn find_user_by_uuid(
        &self,
        uuid: &str,
    ) -> impl std::future::Future<Output = Result<Option<User>, String>> + Send {
        let db = self.db.clone();
        let merchant_id = self.merchant_id.clone();
        let uuid = uuid.to_string();
        async move {
            with_shared_txn(&db, |txn| {
                let merchant_id = merchant_id.clone();
                let uuid = uuid.clone();
                Box::pin(async move {
                    let merchant_uuid = parse_merchant_uuid(&merchant_id)?;
                    let uuid =
                        Uuid::parse_str(&uuid).map_err(|e| format!("解析uuid失败: {}", e))?;
                    let user = Entity::find()
                        .filter(Column::MerchantUuid.eq(merchant_uuid))
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

    fn find_user_by_username(
        &self,
        username: &str,
    ) -> impl std::future::Future<Output = Result<Option<User>, String>> + Send {
        let db = self.db.clone();
        let username = username.to_string();
        async move {
            with_shared_txn(&db, |txn| {
                let username = username.clone();
                Box::pin(async move {
                    let user = Entity::find()
                        .filter(Column::UserName.eq(username))
                        .one(txn)
                        .await
                        .map_err(|e| format!("查询用户失败: {}", e))?;

                    Ok(user.map(UserMapper::to_domain))
                })
            })
            .await
        }
    }

    fn find_user_by_email(
        &self,
        email: &str,
    ) -> impl std::future::Future<Output = Result<Option<User>, String>> + Send {
        let db = self.db.clone();
        let email = email.to_string();
        async move {
            with_shared_txn(&db, |txn| {
                let email = email.clone();
                Box::pin(async move {
                    let user = Entity::find()
                        .filter(Column::Email.eq(email))
                        .one(txn)
                        .await
                        .map_err(|e| format!("查询用户失败: {}", e))?;

                    Ok(user.map(UserMapper::to_domain))
                })
            })
            .await
        }
    }

    fn find_user_by_phone_number(
        &self,
        phone_number: &str,
    ) -> impl std::future::Future<Output = Result<Option<User>, String>> + Send {
        let db = self.db.clone();
        let phone_number = phone_number.to_string();
        async move {
            with_shared_txn(&db, |txn| {
                let phone_number = phone_number.clone();
                Box::pin(async move {
                    let user = Entity::find()
                        .filter(Column::PhoneNumber.eq(phone_number))
                        .one(txn)
                        .await
                        .map_err(|e| format!("查询用户失败: {}", e))?;

                    Ok(user.map(UserMapper::to_domain))
                })
            })
            .await
        }
    }
}

fn parse_merchant_uuid(merchant_id: &str) -> Result<Uuid, String> {
    Uuid::parse_str(merchant_id).map_err(|e| format!("invalid merchant uuid: {}", e))
}
