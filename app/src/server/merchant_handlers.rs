use leptos::prelude::*;
use server_fn::ServerFnError;
use shared::merchant::{ProvisionMerchantRequest, ProvisionMerchantResponse};

#[server(
    name = ProvisionMerchant,
    prefix = "/api",
    endpoint = "/register_merchant",
)]
pub async fn provision_merchant(
    request: ProvisionMerchantRequest,
) -> Result<ProvisionMerchantResponse, ServerFnError> {
    use backend::application::commands::merchant_service::MerchantProvisioningService;
    use backend::infrastructure::db::Database;
    use backend::infrastructure::repositories::merchant_repository_impl::SeaOrmMerchantRepository;

    let pool = expect_context::<Database>();
    let db = pool.get_connection().clone();
    let repository = SeaOrmMerchantRepository::new(db.clone());
    let service = MerchantProvisioningService::new(repository, db)
        .map_err(|e| ServerFnError::new(e))?;

    let merchant = service
        .provision(request)
        .await
        .map_err(|e| ServerFnError::new(e))?;

    Ok(ProvisionMerchantResponse {
        merchant_uuid: merchant.uuid,
        schema_name: merchant.schema_name,
        status: merchant.status,
    })
}
