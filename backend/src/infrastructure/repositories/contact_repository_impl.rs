use sea_orm::entity::prelude::*;

use crate::{
    domain::{
        models::{
            contact::{Contact, CustomerStatus, UpdateContact},
            pagination::Pagination,
        },
        repositories::contact::ContactRepository,
        specifications::contact_spec::{ContactSpecification, SortDirection, SortOption},
    },
    infrastructure::entity::contacts::{Column, Entity},
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

    fn all_contacts(
        &self,
        spec: ContactSpecification,
    ) -> impl std::future::Future<Output = Result<Vec<Contact>, String>> + Send {
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

            let contacts = query
                .all(&self.db)
                .await
                .map_err(|e| format!("获取数据失败: {}", e))?;

            let contacts: Vec<Contact> =
                contacts.into_iter().map(ContactMapper::to_domain).collect();

            Ok(contacts)
        }
    }

    fn get_contact(
        &self,
        uuid: String,
    ) -> impl std::future::Future<Output = Result<Option<Contact>, String>> + Send {
        async move {
            let uuid = Uuid::parse_str(&uuid).expect("解析uuid失败！");
            let contact = Entity::find()
                .filter(Column::ContactUuid.eq(uuid))
                .one(&self.db)
                .await
                .map_err(|e| format!("查询失败: {}", e))?
                .map(|item| ContactMapper::to_domain(item));
            Ok(contact)
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
