use chrono::{DateTime, NaiveDate, NaiveDateTime, TimeZone, Utc};
use sea_orm::{
    ColumnTrait, Condition, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder, QuerySelect,
};
use sea_orm::entity::prelude::*;
use std::collections::{HashMap, HashSet};

use crate::domain::queries::service_request::ServiceRequestQuery as DomainServiceRequestQuery;
use crate::infrastructure::entity::contacts::{Column as ContactColumn, Entity as ContactEntity};
use crate::infrastructure::entity::service_requests::{Column, Entity};
use crate::infrastructure::entity::users::{Column as UserColumn, Entity as UserEntity};
use crate::infrastructure::mappers::service_request_mapper::ServiceRequestMapper;
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

                    if let Some(contact_uuid) = query.contact_uuid {
                        if !contact_uuid.is_empty() {
                            let contact_uuid = Uuid::parse_str(&contact_uuid)
                                .map_err(|e| format!("invalid contact uuid: {}", e))?;
                            condition = condition.add(Column::ContactUuid.eq(contact_uuid));
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

                    let contact_ids: HashSet<Uuid> =
                        models.iter().map(|model| model.contact_uuid).collect();
                    let user_ids: HashSet<Uuid> =
                        models.iter().map(|model| model.creator_uuid).collect();

                    let mut contact_map: HashMap<Uuid, String> = HashMap::new();
                    if !contact_ids.is_empty() {
                        let contacts = ContactEntity::find()
                            .filter(ContactColumn::ContactUuid.is_in(contact_ids.clone()))
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

                    let items = models
                        .into_iter()
                        .map(|model| {
                            let contact_uuid = model.contact_uuid;
                            let creator_uuid = model.creator_uuid;
                            let mut view = ServiceRequestMapper::to_view(model);
                            view.contact_name = contact_map.get(&contact_uuid).cloned();
                            view.creator_name = user_map.get(&creator_uuid).cloned();
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
                        .filter(ContactColumn::ContactUuid.eq(model.contact_uuid))
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

                    let mut view = ServiceRequestMapper::to_view(model);
                    view.contact_name = contact_name;
                    view.creator_name = creator_name;
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
