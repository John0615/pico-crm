use super::model::{Merchant, MerchantUpdate};

pub trait MerchantRepository: Send + Sync {
    fn create_merchant(
        &self,
        merchant: Merchant,
    ) -> impl std::future::Future<Output = Result<Merchant, String>> + Send;

    fn find_by_contact_phone(
        &self,
        contact_phone: &str,
    ) -> impl std::future::Future<Output = Result<Option<Merchant>, String>> + Send;

    fn update_status(
        &self,
        uuid: &str,
        status: String,
    ) -> impl std::future::Future<Output = Result<(), String>> + Send;

    fn update_merchant(
        &self,
        uuid: &str,
        update: MerchantUpdate,
    ) -> impl std::future::Future<Output = Result<Merchant, String>> + Send;
}
