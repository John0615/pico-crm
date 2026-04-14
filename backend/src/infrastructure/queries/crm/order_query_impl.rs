use chrono::{DateTime, NaiveDate, NaiveDateTime, TimeZone, Utc};
use sea_orm::entity::prelude::*;
use sea_orm::{
    ColumnTrait, Condition, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder, QuerySelect,
};
use std::collections::{HashMap, HashSet};

use crate::domain::crm::order::OrderQuery as DomainOrderQuery;
use crate::infrastructure::entity::contacts::{Column as ContactColumn, Entity as ContactEntity};
use crate::infrastructure::entity::orders::{Column, Entity};
use crate::infrastructure::mappers::crm::order_mapper::OrderMapper;
use crate::infrastructure::tenant::with_tenant_txn;
use shared::order::Order as SharedOrder;
use shared::order::OrderQuery;

pub struct SeaOrmOrderQuery {
    db: DatabaseConnection,
    schema_name: String,
}

impl SeaOrmOrderQuery {
    pub fn new(db: DatabaseConnection, schema_name: String) -> Self {
        Self { db, schema_name }
    }
}

impl DomainOrderQuery for SeaOrmOrderQuery {
    type Result = SharedOrder;

    fn list_orders(
        &self,
        query: OrderQuery,
    ) -> impl std::future::Future<Output = Result<(Vec<Self::Result>, u64), String>> + Send {
        let db = self.db.clone();
        let schema_name = self.schema_name.clone();
        async move {
            with_tenant_txn(&db, &schema_name, |txn| {
                Box::pin(async move {
                    let mut select = Entity::find();
                    let mut condition = Condition::all();

                    if let Some(status) = query.status {
                        if !status.is_empty() {
                            condition = condition.add(Column::Status.eq(status));
                        }
                    }

                    if let Some(customer_uuid) = query.customer_uuid {
                        if !customer_uuid.is_empty() {
                            let customer_uuid = Uuid::parse_str(&customer_uuid)
                                .map_err(|e| format!("invalid customer uuid: {}", e))?;
                            condition = condition.add(Column::CustomerUuid.eq(customer_uuid));
                        }
                    }

                    if let Some(start) = query.start_date.as_deref().and_then(parse_datetime) {
                        condition = condition.add(Column::ScheduledStartAt.gte(start));
                    }
                    if let Some(end) = query.end_date.as_deref().and_then(parse_datetime) {
                        condition = condition.add(Column::ScheduledStartAt.lte(end));
                    }

                    select = select.filter(condition);

                    let total = select
                        .clone()
                        .count(txn)
                        .await
                        .map_err(|e| format!("query orders count error: {}", e))?;

                    let models = select
                        .order_by_desc(Column::InsertedAt)
                        .offset(Some((query.page - 1) * query.page_size))
                        .limit(Some(query.page_size))
                        .all(txn)
                        .await
                        .map_err(|e| format!("query orders error: {}", e))?;

                    let customer_ids: HashSet<Uuid> = models
                        .iter()
                        .filter_map(|model| model.customer_uuid)
                        .collect();

                    let mut customer_map: HashMap<Uuid, String> = HashMap::new();
                    if !customer_ids.is_empty() {
                        let customers = ContactEntity::find()
                            .filter(ContactColumn::ContactUuid.is_in(customer_ids.clone()))
                            .all(txn)
                            .await
                            .map_err(|e| format!("query customers error: {}", e))?;
                        for customer in customers {
                            customer_map.insert(customer.contact_uuid, customer.user_name);
                        }
                    }

                    let items = models
                        .into_iter()
                        .map(|model| {
                            let customer_uuid = model.customer_uuid;
                            let mut view = OrderMapper::to_view(model);
                            if let Some(customer_uuid) = customer_uuid {
                                view.customer_name = customer_map.get(&customer_uuid).cloned();
                            }
                            view
                        })
                        .collect();

                    Ok((items, total))
                })
            })
            .await
        }
    }

    fn get_order(
        &self,
        uuid: String,
    ) -> impl std::future::Future<Output = Result<Option<Self::Result>, String>> + Send {
        let db = self.db.clone();
        let schema_name = self.schema_name.clone();
        async move {
            with_tenant_txn(&db, &schema_name, |txn| {
                Box::pin(async move {
                    let order_uuid =
                        Uuid::parse_str(&uuid).map_err(|e| format!("invalid order uuid: {}", e))?;
                    let model = Entity::find_by_id(order_uuid)
                        .one(txn)
                        .await
                        .map_err(|e| format!("query order error: {}", e))?;
                    let Some(model) = model else {
                        return Ok(None);
                    };

                    let customer_name = match model.customer_uuid {
                        Some(customer_uuid) => ContactEntity::find()
                            .filter(ContactColumn::ContactUuid.eq(customer_uuid))
                            .one(txn)
                            .await
                            .map_err(|e| format!("query customer error: {}", e))?
                            .map(|customer| customer.user_name),
                        None => None,
                    };

                    let mut view = OrderMapper::to_view(model);
                    view.customer_name = customer_name;
                    Ok(Some(view))
                })
            })
            .await
        }
    }
}

fn parse_datetime(value: &str) -> Option<DateTime<Utc>> {
    if value.trim().is_empty() {
        return None;
    }
    if let Ok(dt) = DateTime::parse_from_rfc3339(value) {
        return Some(dt.with_timezone(&Utc));
    }
    if let Ok(dt) = NaiveDateTime::parse_from_str(value, "%Y-%m-%d %H:%M:%S") {
        return Some(Utc.from_utc_datetime(&dt));
    }
    if let Ok(date) = NaiveDate::parse_from_str(value, "%Y-%m-%d") {
        if let Some(dt) = date.and_hms_opt(0, 0, 0) {
            return Some(Utc.from_utc_datetime(&dt));
        }
    }
    None
}
