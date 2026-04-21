use async_trait::async_trait;
use sea_orm::{ActiveModelTrait, DatabaseConnection};

use crate::domain::crm::contact::{
    ContactFollowRecord, ContactFollowRecordRepository, CreateContactFollowRecord,
};
use crate::infrastructure::mappers::crm::contact_follow_record_mapper::ContactFollowRecordMapper;
use crate::infrastructure::tenant::with_tenant_txn;

pub struct SeaOrmContactFollowRecordRepository {
    db: DatabaseConnection,
    schema_name: String,
}

impl SeaOrmContactFollowRecordRepository {
    pub fn new(db: DatabaseConnection, schema_name: String) -> Self {
        Self { db, schema_name }
    }
}

#[async_trait]
impl ContactFollowRecordRepository for SeaOrmContactFollowRecordRepository {
    fn create_follow_record(
        &self,
        record: CreateContactFollowRecord,
    ) -> impl std::future::Future<Output = Result<ContactFollowRecord, String>> + Send {
        let db = self.db.clone();
        let schema_name = self.schema_name.clone();
        async move {
            with_tenant_txn(&db, &schema_name, |txn| {
                Box::pin(async move {
                    let active = ContactFollowRecordMapper::to_active_entity(record)?;
                    let created = active
                        .insert(txn)
                        .await
                        .map_err(|e| format!("create contact follow record error: {}", e))?;

                    Ok(ContactFollowRecordMapper::to_domain(created, None))
                })
            })
            .await
        }
    }
}
