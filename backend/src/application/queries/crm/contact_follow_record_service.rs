use crate::domain::crm::contact::ContactFollowRecordQuery;

pub struct ContactFollowRecordQueryService<Q: ContactFollowRecordQuery> {
    query: Q,
}

impl<Q: ContactFollowRecordQuery> ContactFollowRecordQueryService<Q> {
    pub fn new(query: Q) -> Self {
        Self { query }
    }

    pub async fn fetch_follow_records(
        &self,
        contact_uuid: String,
    ) -> Result<Vec<Q::Result>, String> {
        self.query.list_follow_records(contact_uuid).await
    }
}
