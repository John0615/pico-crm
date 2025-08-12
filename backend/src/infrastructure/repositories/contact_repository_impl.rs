use sea_orm::entity::prelude::*;

use crate::{
    domain::{
        models::contact::{Contact, UpdateContact},
        repositories::contact::ContactRepository,
    },
    infrastructure::entity::contacts::{Column, Entity},
    infrastructure::mappers::contact_mapper::ContactMapper,
};

use async_trait::async_trait;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait};

pub struct SeaOrmContactRepository {
    db: DatabaseConnection,
}

impl SeaOrmContactRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl ContactRepository for SeaOrmContactRepository {
    fn create_contact(
        &self,
        contact: Contact,
    ) -> impl Future<Output = Result<Contact, String>> + Send {
        async move {
            let entity = ContactMapper::to_active_entity(contact); // 转换为 Entity
            let new_entity = entity
                .insert(&self.db)
                .await
                .map_err(|e| format!("create contact database error: {}", e))?;

            let new_contact = ContactMapper::to_domain(new_entity);
            Ok(new_contact)
        }
    }

    fn update_contact(
        &self,
        contact: UpdateContact,
    ) -> impl std::future::Future<Output = Result<Contact, String>> + Send {
        async move {
            // 根据 uuid 查询原始数据
            let uuid = Uuid::parse_str(&contact.uuid).expect("解析uuid失败！");
            let original_contact = Entity::find()
                .filter(Column::ContactUuid.eq(uuid))
                .one(&self.db)
                .await
                .map_err(|e| format!("查询原始数据失败: {}", e))?
                .ok_or_else(|| format!("未找到 uuid 为 {} 的联系人", contact.uuid))?;
            // 转换为 ActiveModel
            let active_contact = ContactMapper::to_update_active_entity(contact, &original_contact);

            // 执行更新
            let updated = active_contact
                .update(&self.db)
                .await
                .map_err(|e| format!("更新失败: {}", e))?;
            let updated = ContactMapper::to_domain(updated);

            Ok(updated)
        }
    }

    fn delete_contact(
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
}
