use crate::domain::models::merchant::Merchant as DomainMerchant;
use crate::domain::queries::merchant::MerchantQuery as MQuery;
use shared::merchant::{MerchantListQuery, MerchantPagedResult, MerchantSummary};

pub struct MerchantQueryService<R: MQuery> {
    query: R,
}

impl<R: MQuery<Result = DomainMerchant>> MerchantQueryService<R> {
    pub fn new(query: R) -> Self {
        Self { query }
    }

    pub async fn list_merchants(
        &self,
        query: MerchantListQuery,
    ) -> Result<MerchantPagedResult<MerchantSummary>, String> {
        let result = self.query.list_merchants(query).await?;
        let items = result
            .items
            .into_iter()
            .map(|merchant| merchant.into())
            .collect();

        Ok(MerchantPagedResult {
            items,
            total: result.total,
        })
    }
}
