use crate::domain::crm::after_sales::AfterSalesCaseQuery;
use shared::after_sales::AfterSalesCase as SharedAfterSalesCase;

pub struct AfterSalesCaseQueryService<Q: AfterSalesCaseQuery<Result = SharedAfterSalesCase>> {
    query: Q,
}

impl<Q: AfterSalesCaseQuery<Result = SharedAfterSalesCase>> AfterSalesCaseQueryService<Q> {
    pub fn new(query: Q) -> Self {
        Self { query }
    }

    pub async fn fetch_cases(
        &self,
        order_uuid: String,
    ) -> Result<Vec<SharedAfterSalesCase>, String> {
        self.query.list_cases(order_uuid).await
    }
}
