use crate::{
    domain::{
        crm::contact::{
            ContactFilters, ContactQuery, ContactSpecification, SortDirection, SortOption,
        },
        shared::pagination::Pagination,
    },
    infrastructure::entity::after_sales_cases::{
        Column as AfterSalesColumn, Entity as AfterSalesEntity,
    },
    infrastructure::entity::after_sales_reworks::{
        Column as AfterSalesReworkColumn, Entity as AfterSalesReworkEntity,
    },
    infrastructure::entity::contacts::{Column, Entity},
    infrastructure::entity::orders::{Column as OrderColumn, Entity as OrderEntity},
    infrastructure::mappers::crm::contact_mapper::ContactMapper,
    infrastructure::tenant::with_tenant_txn,
};
use sea_orm::entity::prelude::*;
use shared::contact::Contact;

use async_trait::async_trait;
use sea_orm::sea_query::{Condition, Expr};
use sea_orm::{DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, Select};
use std::collections::HashMap;

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
            let ContactSpecification { filters, sort } = spec;
            with_tenant_txn(&db, &schema_name, |txn| {
                Box::pin(async move {
                    let query =
                        apply_contact_sorting(apply_contact_filters(Entity::find(), filters), sort);

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
            let ContactSpecification { filters, sort } = spec;
            with_tenant_txn(&db, &schema_name, |txn| {
                Box::pin(async move {
                    let query =
                        apply_contact_sorting(apply_contact_filters(Entity::find(), filters), sort);

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
                    let contact = match contact {
                        Some(mut contact) => {
                            let stats = load_contact_after_sales_stats(txn, &[uuid]).await?;
                            if let Some(stat) = stats.get(&uuid) {
                                contact.after_sales_case_count = Some(stat.after_sales_case_count);
                                contact.complaint_case_count = Some(stat.complaint_case_count);
                                contact.refund_case_count = Some(stat.refund_case_count);
                                contact.rework_count = Some(stat.rework_count);
                            }
                            Some(contact)
                        }
                        None => None,
                    };
                    Ok(contact)
                })
            })
            .await
        }
    }
}

fn apply_contact_filters(query: Select<Entity>, filters: ContactFilters) -> Select<Entity> {
    let mut condition = Condition::all();

    if let Some(name) = filters.name {
        condition = condition.add(Column::UserName.contains(name));
    }
    if let Some(phone) = filters.phone {
        condition = condition.add(Column::PhoneNumber.eq(phone));
    }
    if let Some(keyword) = filters.address_keyword {
        condition = condition.add(
            Condition::any()
                .add(Column::Address.contains(keyword.clone()))
                .add(Column::Community.contains(keyword.clone()))
                .add(Column::Building.contains(keyword)),
        );
    }
    if let Some(tag) = filters.tag {
        let pattern = format!("%\"{}\"%", tag.trim());
        condition = condition.add(Expr::col(Column::Tags).cast_as("text").like(pattern));
    }
    if let Some(follow_up_status) = filters.follow_up_status {
        condition = condition.add(Column::FollowUpStatus.eq(follow_up_status));
    }

    query.filter(condition)
}

fn apply_contact_sorting(mut query: Select<Entity>, sort: Vec<SortOption>) -> Select<Entity> {
    if sort.is_empty() {
        return query.order_by_desc(Column::InsertedAt);
    }

    for sort in sort {
        query = match sort {
            SortOption::ByName(direction) => match direction {
                SortDirection::Asc => query.order_by_asc(Column::UserName),
                SortDirection::Desc => query.order_by_desc(Column::UserName),
            },
        };
    }

    query
}

#[derive(Debug, Clone, Default)]
struct ContactAfterSalesStats {
    after_sales_case_count: u64,
    complaint_case_count: u64,
    refund_case_count: u64,
    rework_count: u64,
}

async fn load_contact_after_sales_stats<C>(
    txn: &C,
    contact_ids: &[Uuid],
) -> Result<HashMap<Uuid, ContactAfterSalesStats>, String>
where
    C: sea_orm::ConnectionTrait,
{
    if contact_ids.is_empty() {
        return Ok(HashMap::new());
    }

    let orders = OrderEntity::find()
        .filter(OrderColumn::CustomerUuid.is_in(contact_ids.iter().copied()))
        .all(txn)
        .await
        .map_err(|e| format!("query contact orders error: {}", e))?;

    let mut order_contact_map = HashMap::<Uuid, Uuid>::new();
    for order in orders {
        if let Some(customer_uuid) = order.customer_uuid {
            order_contact_map.insert(order.uuid, customer_uuid);
        }
    }
    if order_contact_map.is_empty() {
        return Ok(HashMap::new());
    }

    let order_ids = order_contact_map.keys().copied().collect::<Vec<_>>();
    let cases = AfterSalesEntity::find()
        .filter(AfterSalesColumn::OrderUuid.is_in(order_ids))
        .all(txn)
        .await
        .map_err(|e| format!("query contact after sales cases error: {}", e))?;

    let mut stats = HashMap::<Uuid, ContactAfterSalesStats>::new();
    let mut case_contact_map = HashMap::<Uuid, Uuid>::new();
    for case in cases {
        let Some(contact_uuid) = order_contact_map.get(&case.order_uuid).copied() else {
            continue;
        };
        case_contact_map.insert(case.uuid, contact_uuid);
        let entry = stats.entry(contact_uuid).or_default();
        entry.after_sales_case_count += 1;
        if case.case_type == "complaint" {
            entry.complaint_case_count += 1;
        }
        if case
            .refund_amount_cents
            .map(|value| value > 0)
            .unwrap_or(false)
            || case.case_type == "refund"
        {
            entry.refund_case_count += 1;
        }
    }

    if !case_contact_map.is_empty() {
        let case_ids = case_contact_map.keys().copied().collect::<Vec<_>>();
        let reworks = AfterSalesReworkEntity::find()
            .filter(AfterSalesReworkColumn::CaseUuid.is_in(case_ids))
            .all(txn)
            .await
            .map_err(|e| format!("query contact after sales reworks error: {}", e))?;
        for rework in reworks {
            if let Some(contact_uuid) = case_contact_map.get(&rework.case_uuid).copied() {
                let entry = stats.entry(contact_uuid).or_default();
                entry.rework_count += 1;
            }
        }
    }

    Ok(stats)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::{DbBackend, QueryTrait};

    #[test]
    fn generated_sql_contains_extended_contact_filters() {
        let filters = ContactFilters {
            name: Some("张".to_string()),
            phone: Some("13800138000".to_string()),
            address_keyword: Some("望京".to_string()),
            tag: Some("VIP".to_string()),
            follow_up_status: Some("scheduled".to_string()),
        };

        let sql = apply_contact_filters(Entity::find(), filters)
            .build(DbBackend::Postgres)
            .to_string();

        assert!(
            sql.contains(r#""customers"."user_name" LIKE '%张%'"#),
            "sql: {sql}"
        );
        assert!(
            sql.contains(r#""customers"."phone_number" = '13800138000'"#),
            "sql: {sql}"
        );
        assert!(
            sql.contains(r#""customers"."address" LIKE '%望京%'"#),
            "sql: {sql}"
        );
        assert!(
            sql.contains(r#""customers"."community" LIKE '%望京%'"#),
            "sql: {sql}"
        );
        assert!(sql.contains(r#"CAST("tags" AS text) LIKE"#), "sql: {sql}");
        assert!(sql.contains(r#"%\"VIP\"%"#), "sql: {sql}");
        assert!(
            sql.contains(r#""customers"."follow_up_status" = 'scheduled'"#),
            "sql: {sql}"
        );
    }

    #[test]
    fn default_sort_uses_inserted_at_desc() {
        let sql = apply_contact_sorting(Entity::find(), vec![])
            .build(DbBackend::Postgres)
            .to_string();

        assert!(sql.contains(r#"ORDER BY "customers"."inserted_at" DESC"#));
    }
}
