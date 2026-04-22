use crate::domain::crm::after_sales_rework::AfterSalesReworkQuery;
use shared::after_sales::AfterSalesRework as SharedAfterSalesRework;

pub struct AfterSalesReworkQueryService<Q: AfterSalesReworkQuery<Result = SharedAfterSalesRework>> {
    query: Q,
}

impl<Q: AfterSalesReworkQuery<Result = SharedAfterSalesRework>> AfterSalesReworkQueryService<Q> {
    pub fn new(query: Q) -> Self {
        Self { query }
    }

    pub async fn fetch_reworks(
        &self,
        case_uuid: String,
    ) -> Result<Vec<SharedAfterSalesRework>, String> {
        self.query.list_reworks(case_uuid).await
    }
}
