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
use crate::infrastructure::tenant::with_tenant_txn;
use shared::service_request::ServiceRequest as SharedServiceRequest;
use shared::service_request::ServiceRequestQuery;

pub struct SeaOrmServiceRequestQuery {
    db: DatabaseConnection,
    schema_name: String,
}

impl SeaOrmServiceRequestQuery {
    pub fn new(db: DatabaseConnection, schema_name: String) -> Self {
        Self { db, schema_name }
    }
}

impl DomainServiceRequestQuery for SeaOrmServiceRequestQuery {
    type Result = SharedServiceRequest;

    fn list_requests(
        &self,
        query: ServiceRequestQuery,
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
                        condition = condition.add(Column::AppointmentStartAt.gte(start));
                    }
                    if let Some(end) = query.end_date.as_deref().and_then(parse_datetime) {
                        condition = condition.add(Column::AppointmentStartAt.lte(end));
                    }

                    select = select.filter(condition);

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
        let schema_name = self.schema_name.clone();
        async move {
            with_tenant_txn(&db, &schema_name, |txn| {
                Box::pin(async move {
                    let request_uuid = Uuid::parse_str(&uuid)
                        .map_err(|e| format!("invalid request uuid: {}", e))?;
                    let model = Entity::find_by_id(request_uuid)
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
