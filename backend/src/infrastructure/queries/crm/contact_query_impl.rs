use crate::{
    domain::{
        crm::contact::{
            ContactQuery, ContactSpecification, CustomerStatus, SortDirection, SortOption,
        },
        shared::pagination::Pagination,
    },
    infrastructure::entity::contacts::{Column, Entity},
    infrastructure::mappers::crm::contact_mapper::ContactMapper,
    infrastructure::tenant::with_tenant_txn,
};
use sea_orm::entity::prelude::*;
use shared::contact::Contact;

use async_trait::async_trait;
use sea_orm::{DatabaseConnection, EntityTrait, PaginatorTrait, QueryOrder};

pub struct SeaOrmContactQuery {
    db: DatabaseConnection,
    schema_name: String,
}

impl SeaOrmContactQuery {
    pub fn new(db: DatabaseConnection, schema_name: String) -> Self {
        Self { db, schema_name }
    }
}

#[async_trait]
impl ContactQuery for SeaOrmContactQuery {
    type Result = Contact;

    fn contacts(
        &self,
        spec: ContactSpecification,
        pagination: Pagination,
    ) -> impl std::future::Future<Output = Result<(Vec<Self::Result>, u64), String>> + Send {
        let db = self.db.clone();
        let schema_name = self.schema_name.clone();
        async move {
            with_tenant_txn(&db, &schema_name, |txn| {
                Box::pin(async move {
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
                                        SortDirection::Desc => {
                                            query.order_by_desc(Column::UserName)
                                        }
                                    };
                                }
                                SortOption::ByLastContact(direction) => {
                                    query = match direction {
                                        SortDirection::Asc => {
                                            query.order_by_asc(Column::LastContact)
                                        }
                                        SortDirection::Desc => {
                                            query.order_by_desc(Column::LastContact)
                                        }
                                    };
                                }
                            }
                        }
                    }

                    let paginator = query.paginate(txn, pagination.size);

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
                        .map(|contact| ContactMapper::to_view(contact))
                        .collect();

                    Ok((contacts, total))
                })
            })
            .await
        }
    }

    fn all_contacts(
        &self,
        spec: ContactSpecification,
    ) -> impl std::future::Future<Output = Result<Vec<Self::Result>, String>> + Send {
        let db = self.db.clone();
        let schema_name = self.schema_name.clone();
        async move {
            with_tenant_txn(&db, &schema_name, |txn| {
                Box::pin(async move {
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
                                        SortDirection::Desc => {
                                            query.order_by_desc(Column::UserName)
                                        }
                                    };
                                }
                                SortOption::ByLastContact(direction) => {
                                    query = match direction {
                                        SortDirection::Asc => {
                                            query.order_by_asc(Column::LastContact)
                                        }
                                        SortDirection::Desc => {
                                            query.order_by_desc(Column::LastContact)
                                        }
                                    };
                                }
                            }
                        }
                    }

                    let contacts = query
                        .all(txn)
                        .await
                        .map_err(|e| format!("获取数据失败: {}", e))?;

                    let contacts: Vec<Contact> =
                        contacts.into_iter().map(ContactMapper::to_view).collect();

                    Ok(contacts)
                })
            })
            .await
        }
    }

    fn get_contact(
        &self,
        uuid: String,
    ) -> impl std::future::Future<Output = Result<Option<Self::Result>, String>> + Send {
        let db = self.db.clone();
        let schema_name = self.schema_name.clone();
        async move {
            with_tenant_txn(&db, &schema_name, |txn| {
                Box::pin(async move {
                    let uuid = Uuid::parse_str(&uuid).expect("解析uuid失败！");
                    let contact = Entity::find()
                        .filter(Column::ContactUuid.eq(uuid))
                        .one(txn)
                        .await
                        .map_err(|e| format!("查询失败: {}", e))?
                        .map(|item| ContactMapper::to_view(item));
                    Ok(contact)
                })
            })
            .await
        }
    }
}
