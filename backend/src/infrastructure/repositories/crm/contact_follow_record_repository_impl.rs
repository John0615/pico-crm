use async_trait::async_trait;
use sea_orm::{ActiveModelTrait, DatabaseConnection};

use crate::domain::crm::contact::{
    ContactFollowRecord, ContactFollowRecordRepository, CreateContactFollowRecord,
};
use crate::infrastructure::mappers::crm::contact_follow_record_mapper::ContactFollowRecordMapper;
use crate::infrastructure::tenant::{parse_merchant_uuid, with_shared_txn};

pub struct SeaOrmContactFollowRecordRepository {
    db: DatabaseConnection,
    merchant_id: String,
}

impl SeaOrmContactFollowRecordRepository {
    pub fn new(db: DatabaseConnection, merchant_id: String) -> Self {
        Self { db, merchant_id }
    }
}

#[async_trait]
impl ContactFollowRecordRepository for SeaOrmContactFollowRecordRepository {
    fn create_follow_record(
        &self,
        record: CreateContactFollowRecord,
    ) -> impl std::future::Future<Output = Result<ContactFollowRecord, String>> + Send {
        let db = self.db.clone();
        let merchant_id = self.merchant_id.clone();
        async move {
            with_shared_txn(&db, |txn| {
                let merchant_id = merchant_id.clone();
                Box::pin(async move {
                    let merchant_uuid = parse_merchant_uuid(&merchant_id)?;
                    let mut active = ContactFollowRecordMapper::to_active_entity(record)?;
                    active.merchant_id = sea_orm::ActiveValue::Set(Some(merchant_uuid));
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
