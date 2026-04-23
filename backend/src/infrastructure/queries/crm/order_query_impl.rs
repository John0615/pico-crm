use chrono::{DateTime, NaiveDate, NaiveDateTime, TimeZone, Utc};
use sea_orm::entity::prelude::*;
use sea_orm::{
    ColumnTrait, Condition, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder, QuerySelect,
};
use std::collections::{HashMap, HashSet};

use crate::domain::crm::order::OrderQuery as DomainOrderQuery;
use crate::infrastructure::entity::contacts::{Column as ContactColumn, Entity as ContactEntity};
use crate::infrastructure::entity::order_change_logs::{
    Column as OrderChangeLogColumn, Entity as OrderChangeLogEntity,
};
use crate::infrastructure::entity::orders::{Column, Entity};
use crate::infrastructure::entity::service_catalogs::{
    Column as ServiceCatalogColumn, Entity as ServiceCatalogEntity,
};
use crate::infrastructure::entity::service_requests::{
    Column as ServiceRequestColumn, Entity as ServiceRequestEntity,
};
use crate::infrastructure::mappers::crm::order_mapper::OrderMapper;
use crate::infrastructure::tenant::{parse_merchant_uuid, with_shared_txn};
use crate::infrastructure::utils::parse_date_time_to_string;
use shared::order::{Order as SharedOrder, OrderChangeLogDto, OrderQuery};

pub struct SeaOrmOrderQuery {
    db: DatabaseConnection,
    merchant_id: String,
}

impl SeaOrmOrderQuery {
    pub fn new(db: DatabaseConnection, merchant_id: String) -> Self {
        Self { db, merchant_id }
    }
}

impl DomainOrderQuery for SeaOrmOrderQuery {
    type Result = SharedOrder;

    fn list_orders(
        &self,
        query: OrderQuery,
    ) -> impl std::future::Future<Output = Result<(Vec<Self::Result>, u64), String>> + Send {
        let db = self.db.clone();
        let merchant_id = self.merchant_id.clone();
        async move {
            with_shared_txn(&db, |txn| {
                let merchant_id = merchant_id.clone();
                Box::pin(async move {
                    let merchant_uuid = parse_merchant_uuid(&merchant_id)?;
                    let mut select = Entity::find();
                    select = select.filter(build_order_condition(&query, merchant_uuid)?);

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
                    let request_ids: HashSet<Uuid> =
                        models.iter().filter_map(|model| model.request_id).collect();

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

                    let mut request_service_catalog_map: HashMap<Uuid, Option<Uuid>> =
                        HashMap::new();
                    if !request_ids.is_empty() {
                        let requests = ServiceRequestEntity::find()
                            .filter(ServiceRequestColumn::Uuid.is_in(request_ids.clone()))
                            .all(txn)
                            .await
                            .map_err(|e| format!("query service requests error: {}", e))?;
                        for request in requests {
                            request_service_catalog_map
                                .insert(request.uuid, request.service_catalog_uuid);
                        }
                    }

                    let service_catalog_ids: HashSet<Uuid> = request_service_catalog_map
                        .values()
                        .filter_map(|value| *value)
                        .collect();
                    let mut service_catalog_map: HashMap<Uuid, String> = HashMap::new();
                    if !service_catalog_ids.is_empty() {
                        let catalogs = ServiceCatalogEntity::find()
                            .filter(ServiceCatalogColumn::Uuid.is_in(service_catalog_ids.clone()))
                            .all(txn)
                            .await
                            .map_err(|e| format!("query service catalogs error: {}", e))?;
                        for catalog in catalogs {
                            service_catalog_map.insert(catalog.uuid, catalog.name);
                        }
                    }

                    let items = models
                        .into_iter()
                        .map(|model| {
                            let customer_uuid = model.customer_uuid;
                            let request_id = model.request_id;
                            let mut view = OrderMapper::to_view(model);
                            if let Some(customer_uuid) = customer_uuid {
                                view.customer_name = customer_map.get(&customer_uuid).cloned();
                            }
                            if let Some(request_id) = request_id {
                                if let Some(service_catalog_uuid) = request_service_catalog_map
                                    .get(&request_id)
                                    .and_then(|value| *value)
                                {
                                    view.service_catalog_uuid =
                                        Some(service_catalog_uuid.to_string());
                                    view.service_catalog_name =
                                        service_catalog_map.get(&service_catalog_uuid).cloned();
                                }
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
        let merchant_id = self.merchant_id.clone();
        async move {
            with_shared_txn(&db, |txn| {
                let merchant_id = merchant_id.clone();
                Box::pin(async move {
                    let merchant_uuid = parse_merchant_uuid(&merchant_id)?;
                    let order_uuid =
                        Uuid::parse_str(&uuid).map_err(|e| format!("invalid order uuid: {}", e))?;
                    let model = Entity::find()
                        .filter(Column::MerchantId.eq(merchant_uuid))
                        .filter(Column::Uuid.eq(order_uuid))
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

                    let service_catalog = match model.request_id {
                        Some(request_id) => {
                            let request = ServiceRequestEntity::find()
                                .filter(ServiceRequestColumn::Uuid.eq(request_id))
                                .one(txn)
                                .await
                                .map_err(|e| format!("query service request error: {}", e))?;
                            match request.and_then(|value| value.service_catalog_uuid) {
                                Some(service_catalog_uuid) => {
                                    let item = ServiceCatalogEntity::find()
                                        .filter(ServiceCatalogColumn::Uuid.eq(service_catalog_uuid))
                                        .one(txn)
                                        .await
                                        .map_err(|e| {
                                            format!("query service catalog error: {}", e)
                                        })?;
                                    item.map(|item| (service_catalog_uuid, item.name))
                                }
                                None => None,
                            }
                        }
                        None => None,
                    };

                    let mut view = OrderMapper::to_view(model);
                    view.customer_name = customer_name;
                    if let Some((service_catalog_uuid, service_catalog_name)) = service_catalog {
                        view.service_catalog_uuid = Some(service_catalog_uuid.to_string());
                        view.service_catalog_name = Some(service_catalog_name);
                    }
                    Ok(Some(view))
                })
            })
            .await
        }
    }

    fn list_order_change_logs(
        &self,
        uuid: String,
    ) -> impl std::future::Future<Output = Result<Vec<OrderChangeLogDto>, String>> + Send {
        let db = self.db.clone();
        let merchant_id = self.merchant_id.clone();
        async move {
            with_shared_txn(&db, |txn| {
                let merchant_id = merchant_id.clone();
                Box::pin(async move {
                    let merchant_uuid = parse_merchant_uuid(&merchant_id)?;
                    let order_uuid =
                        Uuid::parse_str(&uuid).map_err(|e| format!("invalid order uuid: {}", e))?;
                    let models = OrderChangeLogEntity::find()
                        .filter(OrderChangeLogColumn::MerchantId.eq(merchant_uuid))
                        .filter(OrderChangeLogColumn::OrderUuid.eq(order_uuid))
                        .order_by_desc(OrderChangeLogColumn::CreatedAt)
                        .all(txn)
                        .await
                        .map_err(|e| format!("query order change logs error: {}", e))?;

                    Ok(models
                        .into_iter()
                        .map(|model| OrderChangeLogDto {
                            uuid: model.uuid.to_string(),
                            order_uuid: model.order_uuid.to_string(),
                            action: model.action,
                            operator_uuid: model.operator_uuid.map(|value| value.to_string()),
                            before_data: model.before_data.map(Into::into),
                            after_data: model.after_data.map(Into::into),
                            created_at: parse_date_time_to_string(model.created_at),
                        })
                        .collect())
                })
            })
            .await
        }
    }
}

fn build_order_condition(query: &OrderQuery, merchant_uuid: Uuid) -> Result<Condition, String> {
    let mut condition = Condition::all();
    condition = condition.add(Column::MerchantId.eq(merchant_uuid));

    if let Some(statuses) = query.statuses.clone() {
        let statuses = statuses
            .into_iter()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .collect::<Vec<_>>();
        if !statuses.is_empty() {
            condition = condition.add(Column::Status.is_in(statuses));
        }
    }

    if let Some(status) = query.status.clone() {
        if !status.is_empty() {
            condition = condition.add(Column::Status.eq(status));
        }
    }

    if let Some(settlement_status) = query.settlement_status.clone() {
        if !settlement_status.is_empty() {
            condition = condition.add(Column::SettlementStatus.eq(settlement_status));
        }
    }

    if let Some(customer_uuid) = query.customer_uuid.clone() {
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

    Ok(condition)
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

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::{DbBackend, QueryTrait};

    #[test]
    fn generated_sql_contains_merchant_scope_for_order_list() {
        let merchant_uuid =
            Uuid::parse_str("11111111-1111-1111-1111-111111111111").expect("valid uuid");
        let query = OrderQuery {
            page: 1,
            page_size: 20,
            status: Some("pending".to_string()),
            statuses: None,
            settlement_status: None,
            customer_uuid: None,
            start_date: None,
            end_date: None,
        };

        let sql = Entity::find()
            .filter(build_order_condition(&query, merchant_uuid).expect("condition"))
            .build(DbBackend::Postgres)
            .to_string();

        assert!(
            sql.contains(r#""orders"."merchant_id" = '11111111-1111-1111-1111-111111111111'"#),
            "sql: {sql}"
        );
        assert!(
            sql.contains(r#""orders"."status" = 'pending'"#),
            "sql: {sql}"
        );
    }
}
