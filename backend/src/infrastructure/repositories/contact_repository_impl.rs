use sea_orm::entity::prelude::*;

use crate::{
    domain::{
        models::{
            contact::{Contact, CustomerStatus},
            pagination::Pagination,
        },
        repositories::contact::ContactRepository,
        specifications::contact_spec::{ContactSpecification, SortDirection, SortOption},
    },
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
        spec: ContactSpecification,
        pagination: Pagination,
    ) -> impl std::future::Future<Output = Result<(Vec<Contact>, u64), String>> + Send {
        async move {
            let mut query = Entity::find();
            println!("spec.filters>>>{:?}", spec.filters);
            if let Some(name) = spec.filters.name {
                query = query.filter(Column::UserName.contains(name));
            }
            if let Some(status) = spec.filters.status {
                let status_num = match status {
                    CustomerStatus::Signed => 1,
                    CustomerStatus::Pending => 2,
                    CustomerStatus::Churned => 3,
                };
                query = query.filter(Column::Status.eq(status_num));
            }
            if let Some(email) = spec.filters.email {
                query = query.filter(Column::Email.contains(email));
            }
            if let Some(phone) = spec.filters.phone {
                query = query.filter(Column::PhoneNumber.eq(phone));
            }

            if spec.sort.is_empty() {
                // 默认排序
                query = query.order_by_desc(Column::InsertedAt);
            } else {
                for sort in spec.sort {
                    match sort {
                        SortOption::ByName(direction) => {
                            query = match direction {
                                SortDirection::Asc => query.order_by_asc(Column::UserName),
                                SortDirection::Desc => query.order_by_desc(Column::UserName),
                            };
                        }
                        SortOption::ByLastContact(direction) => {
                            query = match direction {
                                SortDirection::Asc => query.order_by_asc(Column::LastContact),
                                SortDirection::Desc => query.order_by_desc(Column::LastContact),
                            };
                        }
                    }
                }
            }

            let paginator = query.paginate(&self.db, pagination.size);

            let contacts = paginator
                .fetch_page(pagination.page - 1)
                .await
                .map_err(|_| "获取数据失败".to_string())?;

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
