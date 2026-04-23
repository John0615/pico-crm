use sea_orm::entity::prelude::*;

use crate::{
    domain::crm::contact::{Contact, ContactRepository, UpdateContact},
    infrastructure::entity::contacts::{Column, Entity},
    infrastructure::mappers::crm::contact_mapper::ContactMapper,
    infrastructure::tenant::{parse_merchant_uuid, with_shared_txn},
};

use async_trait::async_trait;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait};

pub struct SeaOrmContactRepository {
    db: DatabaseConnection,
    merchant_id: String,
}

impl SeaOrmContactRepository {
    pub fn new(db: DatabaseConnection, merchant_id: String) -> Self {
        Self { db, merchant_id }
    }
}

#[async_trait]
impl ContactRepository for SeaOrmContactRepository {
    fn create_contact(
        &self,
        contact: Contact,
    ) -> impl Future<Output = Result<Contact, String>> + Send {
        let db = self.db.clone();
        let merchant_id = self.merchant_id.clone();
        async move {
            with_shared_txn(&db, |txn| {
                let merchant_id = merchant_id.clone();
                Box::pin(async move {
                    let merchant_uuid = parse_merchant_uuid(&merchant_id)?;
                    let mut entity = ContactMapper::to_active_entity(contact); // 转换为 Entity
                    entity.merchant_id = sea_orm::ActiveValue::Set(Some(merchant_uuid));
                    let new_entity = entity
                        .insert(txn)
                        .await
                        .map_err(|e| format!("create contact database error: {}", e))?;

                    let new_contact = ContactMapper::to_domain(new_entity);
                    Ok(new_contact)
                })
            })
            .await
        }
    }

    fn update_contact(
        &self,
        contact: UpdateContact,
    ) -> impl std::future::Future<Output = Result<Contact, String>> + Send {
        let db = self.db.clone();
        let merchant_id = self.merchant_id.clone();
        async move {
            with_shared_txn(&db, |txn| {
                let merchant_id = merchant_id.clone();
                Box::pin(async move {
                    let merchant_uuid = parse_merchant_uuid(&merchant_id)?;
                    // 根据 uuid 查询原始数据
                    let uuid = Uuid::parse_str(&contact.uuid).expect("解析uuid失败！");
                    let original_contact = Entity::find()
                        .filter(Column::MerchantId.eq(merchant_uuid))
                        .filter(Column::ContactUuid.eq(uuid))
                        .one(txn)
                        .await
                        .map_err(|e| format!("查询原始数据失败: {}", e))?
                        .ok_or_else(|| format!("未找到 uuid 为 {} 的联系人", contact.uuid))?;
                    // 转换为 ActiveModel
                    let active_contact =
                        ContactMapper::to_update_active_entity(contact, &original_contact);

                    // 执行更新
                    let updated = active_contact
                        .update(txn)
                        .await
                        .map_err(|e| format!("更新失败: {}", e))?;
                    let updated = ContactMapper::to_domain(updated);

                    Ok(updated)
                })
            })
            .await
        }
    }

    fn delete_contact(
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
                        .filter(Column::MerchantId.eq(merchant_uuid))
                        .filter(Column::ContactUuid.eq(uuid))
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

    fn find_contact_by_phone_number(
        &self,
        phone_number: &str,
    ) -> impl std::future::Future<Output = Result<Option<Contact>, String>> + Send {
        let db = self.db.clone();
        let merchant_id = self.merchant_id.clone();
        let phone_number = phone_number.to_string();
        async move {
            with_shared_txn(&db, |txn| {
                let merchant_id = merchant_id.clone();
                let phone_number = phone_number.clone();
                Box::pin(async move {
                    let merchant_uuid = parse_merchant_uuid(&merchant_id)?;
                    let contact = Entity::find()
                        .filter(Column::MerchantId.eq(merchant_uuid))
                        .filter(Column::PhoneNumber.eq(phone_number))
                        .one(txn)
                        .await
                        .map_err(|e| format!("查询客户失败: {}", e))?;

                    Ok(contact.map(ContactMapper::to_domain))
                })
            })
            .await
        }
    }
}
