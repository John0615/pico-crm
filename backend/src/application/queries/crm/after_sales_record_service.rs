use crate::domain::crm::after_sales::AfterSalesCaseQuery;
use shared::after_sales::AfterSalesCaseRecord as SharedAfterSalesCaseRecord;

pub struct AfterSalesCaseRecordQueryService<
    Q: AfterSalesCaseQuery<RecordResult = SharedAfterSalesCaseRecord>,
> {
    query: Q,
}

impl<Q: AfterSalesCaseQuery<RecordResult = SharedAfterSalesCaseRecord>>
    AfterSalesCaseRecordQueryService<Q>
{
    pub fn new(query: Q) -> Self {
        Self { query }
    }

    pub async fn fetch_records(
        &self,
        case_uuid: String,
    ) -> Result<Vec<SharedAfterSalesCaseRecord>, String> {
        self.query.list_case_records(case_uuid).await
    }
}
