use crate::domain::crm::after_sales::{AfterSalesCaseRepository, CreateAfterSalesCaseRecord};
use shared::after_sales::{
    AfterSalesCaseRecord as SharedAfterSalesCaseRecord, CreateAfterSalesCaseRecordRequest,
};

pub struct AfterSalesCaseRecordAppService<R: AfterSalesCaseRepository> {
    repo: R,
}

impl<R: AfterSalesCaseRepository> AfterSalesCaseRecordAppService<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    pub async fn create_record(
        &self,
        case_uuid: String,
        payload: CreateAfterSalesCaseRecordRequest,
        operator_uuid: Option<String>,
    ) -> Result<SharedAfterSalesCaseRecord, String> {
        let record = CreateAfterSalesCaseRecord {
            case_uuid,
            operator_uuid,
            content: payload.content,
            status: payload.status,
        };
        record.verify()?;

        let created = self.repo.create_case_record(record).await?;
        Ok(created.into())
    }
}
