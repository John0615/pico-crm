use sea_orm::entity::prelude::*;

use crate::{
    domain::{models::contact::Contact, repositories::contact::ContactRepository},
    entity::contacts::{Column, Entity},
    infrastructure::mappers::contact_mapper::ContactMapper,
};
use async_trait::async_trait;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryOrder};

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

    fn contacts(
        &self,
        page: u64,
        page_size: u64,
    ) -> impl std::future::Future<Output = Result<(Vec<Contact>, u64), String>> + Send {
        async move {
            let paginator = Entity::find()
                .order_by_desc(Column::InsertedAt)
                .paginate(&self.db, page_size); // 每页10条
            // 获取当前页数据
            let contacts = paginator
                .fetch_page(page - 1) // 第一页（页码从0开始）
                .await
                .map_err(|_| "获取数据失败".to_string())?;
            // 获取总数
            let total = paginator
                .num_items()
                .await
                .map_err(|_| "获取总数失败".to_string())?;
            let contacts: Vec<Contact> = contacts
                .into_iter()
                .map(|contact| ContactMapper::to_domain(contact))
                .collect();
            Ok((contacts, total))
        }
    }

    // async fn update(&self, contact: Contact) -> Result<Contact, String> {
    //     let model = ContactEntity::from(contact);
    //     let model = model.update(&self.db).await.map_err(|e| e.to_string())?;
    //     Ok(Contact::from(model))
    // }

    // async fn delete(&self, id: i32) -> Result<(), String> {
    //     let model = ContactEntity::find_by_id(id)
    //         .one(&self.db)
    //         .await
    //         .map_err(|e| e.to_string())?;
    //     model
    //         .ok_or("Contact not found".to_string())
    //         .map(|model| model.delete(&self.db).await.map_err(|e| e.to_string()))
    // }

    // async fn list(&self, page: u32, per_page: u32) -> Result<Vec<Contact>, String> {
    //     let models = ContactEntity::find()
    //         .order_by_asc(ContactEntity::Id)
    //         .paginate(&self.db, per_page)
    //         .fetch_page(page)
    //         .await
    //         .map_err(|e| e.to_string())?;
    //     Ok(models.into_iter().map(Contact::from).collect())
    // }
}
