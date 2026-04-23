use chrono::{DateTime, NaiveDate, NaiveDateTime, TimeZone, Utc};
use sea_orm::entity::prelude::*;
use sea_orm::{
    ColumnTrait, Condition, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder, QuerySelect,
};
use std::collections::{HashMap, HashSet};

use crate::domain::crm::service_request::ServiceRequestQuery as DomainServiceRequestQuery;
use crate::infrastructure::entity::contacts::{Column as ContactColumn, Entity as ContactEntity};
use crate::infrastructure::entity::service_catalogs::{
    Column as ServiceCatalogColumn, Entity as ServiceCatalogEntity,
};
use crate::infrastructure::entity::service_requests::{Column, Entity};
use crate::infrastructure::entity::users::{Column as UserColumn, Entity as UserEntity};
use crate::infrastructure::mappers::crm::service_request_mapper::ServiceRequestMapper;
use crate::infrastructure::tenant::{parse_merchant_uuid, with_shared_txn};
use shared::service_request::ServiceRequest as SharedServiceRequest;
use shared::service_request::ServiceRequestQuery;

pub struct SeaOrmServiceRequestQuery {
    db: DatabaseConnection,
    merchant_id: String,
}

impl SeaOrmServiceRequestQuery {
    pub fn new(db: DatabaseConnection, merchant_id: String) -> Self {
        Self { db, merchant_id }
    }
}

impl DomainServiceRequestQuery for SeaOrmServiceRequestQuery {
    type Result = SharedServiceRequest;

    fn list_requests(
        &self,
        query: ServiceRequestQuery,
    ) -> impl std::future::Future<Output = Result<(Vec<Self::Result>, u64), String>> + Send {
        let db = self.db.clone();
        let merchant_id = self.merchant_id.clone();
        async move {
            with_shared_txn(&db, |txn| {
                let merchant_id = merchant_id.clone();
                Box::pin(async move {
                    let merchant_uuid = parse_merchant_uuid(&merchant_id)?;
                    let mut select = Entity::find();
                    select = select.filter(build_service_request_condition(&query, merchant_uuid)?);

                    let total = select
                        .clone()
                        .count(txn)
                        .await
                        .map_err(|e| format!("query service request count error: {}", e))?;

                    let models = select
                        .order_by_desc(Column::InsertedAt)
                        .offset(Some((query.page - 1) * query.page_size))
                        .limit(Some(query.page_size))
                        .all(txn)
                        .await
                        .map_err(|e| format!("query service requests error: {}", e))?;

                    let customer_ids: HashSet<Uuid> =
                        models.iter().map(|model| model.customer_uuid).collect();
                    let user_ids: HashSet<Uuid> =
                        models.iter().map(|model| model.creator_uuid).collect();
                    let service_catalog_ids: HashSet<Uuid> = models
                        .iter()
                        .filter_map(|model| model.service_catalog_uuid)
                        .collect();

                    let mut contact_map: HashMap<Uuid, String> = HashMap::new();
                    if !customer_ids.is_empty() {
                        let contacts = ContactEntity::find()
                            .filter(ContactColumn::ContactUuid.is_in(customer_ids.clone()))
                            .all(txn)
                            .await
                            .map_err(|e| format!("query contacts error: {}", e))?;
                        for contact in contacts {
                            contact_map.insert(contact.contact_uuid, contact.user_name);
                        }
                    }

                    let mut user_map: HashMap<Uuid, String> = HashMap::new();
                    if !user_ids.is_empty() {
                        let users = UserEntity::find()
                            .filter(UserColumn::Uuid.is_in(user_ids.clone()))
                            .all(txn)
                            .await
                            .map_err(|e| format!("query users error: {}", e))?;
                        for user in users {
                            user_map.insert(user.uuid, user.user_name);
                        }
                    }

                    let mut service_catalog_map: HashMap<Uuid, String> = HashMap::new();
                    if !service_catalog_ids.is_empty() {
                        let items = ServiceCatalogEntity::find()
                            .filter(ServiceCatalogColumn::Uuid.is_in(service_catalog_ids.clone()))
                            .all(txn)
                            .await
                            .map_err(|e| format!("query service catalogs error: {}", e))?;
                        for item in items {
                            service_catalog_map.insert(item.uuid, item.name);
                        }
                    }

                    let items = models
                        .into_iter()
                        .map(|model| {
                            let customer_uuid = model.customer_uuid;
                            let creator_uuid = model.creator_uuid;
                            let service_catalog_uuid = model.service_catalog_uuid;
                            let mut view = ServiceRequestMapper::to_view(model);
                            view.contact_name = contact_map.get(&customer_uuid).cloned();
                            view.creator_name = user_map.get(&creator_uuid).cloned();
                            view.service_catalog_name = service_catalog_uuid
                                .and_then(|uuid| service_catalog_map.get(&uuid).cloned());
                            view
                        })
                        .collect();

                    Ok((items, total))
                })
            })
            .await
        }
    }

    fn get_request(
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
                    let request_uuid = Uuid::parse_str(&uuid)
                        .map_err(|e| format!("invalid request uuid: {}", e))?;
                    let model = Entity::find()
                        .filter(Column::MerchantId.eq(merchant_uuid))
                        .filter(Column::Uuid.eq(request_uuid))
                        .one(txn)
                        .await
                        .map_err(|e| format!("query service request error: {}", e))?;
                    let Some(model) = model else {
                        return Ok(None);
                    };

                    let contact_name = ContactEntity::find()
                        .filter(ContactColumn::ContactUuid.eq(model.customer_uuid))
                        .one(txn)
                        .await
                        .map_err(|e| format!("query contact error: {}", e))?
                        .map(|contact| contact.user_name);

                    let creator_name = UserEntity::find()
                        .filter(UserColumn::Uuid.eq(model.creator_uuid))
                        .one(txn)
                        .await
                        .map_err(|e| format!("query user error: {}", e))?
                        .map(|user| user.user_name);

                    let service_catalog_name = match model.service_catalog_uuid {
                        Some(service_catalog_uuid) => ServiceCatalogEntity::find()
                            .filter(ServiceCatalogColumn::Uuid.eq(service_catalog_uuid))
                            .one(txn)
                            .await
                            .map_err(|e| format!("query service catalog error: {}", e))?
                            .map(|item| item.name),
                        None => None,
                    };

                    let mut view = ServiceRequestMapper::to_view(model);
                    view.contact_name = contact_name;
                    view.creator_name = creator_name;
                    view.service_catalog_name = service_catalog_name;
                    Ok(Some(view))
                })
            })
            .await
        }
    }
}

fn build_service_request_condition(
    query: &ServiceRequestQuery,
    merchant_uuid: Uuid,
) -> Result<Condition, String> {
    let mut condition = Condition::all();
    condition = condition.add(Column::MerchantId.eq(merchant_uuid));

    if let Some(status) = query.status.clone() {
        if !status.is_empty() {
            condition = condition.add(Column::Status.eq(status));
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
        condition = condition.add(Column::AppointmentStartAt.gte(start));
    }
    if let Some(end) = query.end_date.as_deref().and_then(parse_datetime) {
        condition = condition.add(Column::AppointmentStartAt.lte(end));
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
    fn generated_sql_contains_merchant_scope_for_service_request_list() {
        let merchant_uuid =
            Uuid::parse_str("11111111-1111-1111-1111-111111111111").expect("valid uuid");
        let query = ServiceRequestQuery {
            page: 1,
            page_size: 20,
            status: Some("new".to_string()),
            customer_uuid: None,
            start_date: None,
            end_date: None,
        };

        let sql = Entity::find()
            .filter(build_service_request_condition(&query, merchant_uuid).expect("condition"))
            .build(DbBackend::Postgres)
            .to_string();

        assert!(
            sql.contains(
                r#""service_requests"."merchant_id" = '11111111-1111-1111-1111-111111111111'"#
            ),
            "sql: {sql}"
        );
        assert!(
            sql.contains(r#""service_requests"."status" = 'new'"#),
            "sql: {sql}"
        );
    }
}
