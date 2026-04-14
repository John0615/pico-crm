use crate::domain::platform::merchant::{Merchant, MerchantRepository, MerchantUpdate};

pub struct AdminMerchantService<R: MerchantRepository> {
    repo: R,
}

impl<R: MerchantRepository> AdminMerchantService<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    pub async fn update_merchant(
        &self,
        uuid: &str,
        update: MerchantUpdate,
    ) -> Result<Merchant, String> {
        self.repo.update_merchant(uuid, update).await
    }
}
