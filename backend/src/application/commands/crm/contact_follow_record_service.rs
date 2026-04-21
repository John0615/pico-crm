use crate::domain::crm::contact::{ContactFollowRecordRepository, CreateContactFollowRecord};
use shared::contact::{
    ContactFollowRecord as SharedContactFollowRecord,
    CreateContactFollowRecordRequest,
};

pub struct ContactFollowRecordAppService<R: ContactFollowRecordRepository> {
    repo: R,
}

impl<R: ContactFollowRecordRepository> ContactFollowRecordAppService<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    pub async fn create_follow_record(
        &self,
        payload: CreateContactFollowRecordRequest,
        operator_uuid: Option<String>,
    ) -> Result<SharedContactFollowRecord, String> {
        let record: CreateContactFollowRecord = (payload, operator_uuid).try_into()?;
        record.verify()?;

        let created = self.repo.create_follow_record(record).await?;
        Ok(created.into())
    }
}
