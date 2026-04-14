use crate::domain::crm::order::OrderQuery as DomainOrderQuery;
use shared::ListResult;
use shared::order::{Order, OrderQuery};

pub struct OrderQueryService<R: DomainOrderQuery> {
    query: R,
}

impl<R: DomainOrderQuery<Result = Order>> OrderQueryService<R> {
    pub fn new(query: R) -> Self {
        Self { query }
    }

    pub async fn fetch_orders(&self, params: OrderQuery) -> Result<ListResult<Order>, String> {
        let (items, total) = self.query.list_orders(params).await?;
        Ok(ListResult { items, total })
    }

    pub async fn fetch_order(&self, uuid: String) -> Result<Option<Order>, String> {
        self.query.get_order(uuid).await
    }
}
