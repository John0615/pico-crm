use shared::merchant::{MerchantListQuery, MerchantPagedResult};

pub trait MerchantQuery: Send + Sync {
    type Result;

    fn list_merchants(
        &self,
        query: MerchantListQuery,
    ) -> impl std::future::Future<Output = Result<MerchantPagedResult<Self::Result>, String>> + Send;

    fn find_by_uuid(
        &self,
        uuid: &str,
    ) -> impl std::future::Future<Output = Result<Option<Self::Result>, String>> + Send;
}
