use leptos::prelude::*;
use server_fn::ServerFnError;
use shared::merchant::{
    MerchantListQuery, MerchantPagedResult, MerchantSummary, ProvisionMerchantRequest,
    UpdateMerchantRequest,
};
use chrono::{DateTime, FixedOffset, NaiveDateTime, Utc, TimeZone};

#[server(
    name = FetchMerchants,
    prefix = "/api/admin",
    endpoint = "/merchants",
)]
pub async fn fetch_merchants(
    params: MerchantListQuery,
) -> Result<MerchantPagedResult<MerchantSummary>, ServerFnError> {
    use backend::application::queries::merchant_service::MerchantQueryService;
    use backend::infrastructure::db::Database;
    use backend::infrastructure::queries::merchant_query_impl::SeaOrmMerchantQuery;

    let pool = expect_context::<Database>();
    let db = pool.get_connection().clone();

    let query_repository = SeaOrmMerchantQuery::new(db);
    let query_service = MerchantQueryService::new(query_repository);

    let result = query_service
        .list_merchants(params)
        .await
        .map_err(|e| ServerFnError::new(format!("查询商户失败: {}", e)))?;

    Ok(result)
}

#[server(
    name = CreateMerchant,
    prefix = "/api/admin",
    endpoint = "/merchants/create",
)]
pub async fn create_merchant(
    request: ProvisionMerchantRequest,
) -> Result<MerchantSummary, ServerFnError> {
    use backend::application::commands::merchant_service::MerchantProvisioningService;
    use backend::infrastructure::db::Database;
    use backend::infrastructure::repositories::merchant_repository_impl::SeaOrmMerchantRepository;

    let pool = expect_context::<Database>();
    let db = pool.get_connection().clone();
    let repository = SeaOrmMerchantRepository::new(db.clone());
    let service = MerchantProvisioningService::new(repository, db)
        .map_err(|e| ServerFnError::new(format!("初始化失败: {}", e)))?;

    let merchant = service
        .provision(request)
        .await
        .map_err(|e| ServerFnError::new(format!("创建商户失败: {}", e)))?;

    Ok(merchant.into())
}

#[server(
    name = UpdateMerchant,
    prefix = "/api/admin",
    endpoint = "/merchants/update",
)]
pub async fn update_merchant(
    uuid: String,
    request: UpdateMerchantRequest,
) -> Result<MerchantSummary, ServerFnError> {
    use backend::application::commands::admin_merchant_service::AdminMerchantService;
    use backend::domain::models::merchant::MerchantUpdate;
    use backend::infrastructure::db::Database;
    use backend::infrastructure::repositories::merchant_repository_impl::SeaOrmMerchantRepository;
    let pool = expect_context::<Database>();
    let db = pool.get_connection().clone();
    let repo = SeaOrmMerchantRepository::new(db);
    let service = AdminMerchantService::new(repo);

    let update = MerchantUpdate {
        name: normalize_optional_string(request.name),
        short_name: normalize_optional_string(request.short_name),
        contact_name: normalize_optional_string(request.contact_name),
        contact_phone: normalize_optional_string(request.contact_phone),
        merchant_type: normalize_optional_string(request.merchant_type),
        status: normalize_optional_string(request.status),
        plan_type: normalize_optional_string(request.plan_type),
        trial_end_at: parse_optional_datetime(request.trial_end_at)?,
        expired_at: parse_optional_datetime(request.expired_at)?,
    };

    let merchant = service
        .update_merchant(&uuid, update)
        .await
        .map_err(|e| ServerFnError::new(format!("更新商户失败: {}", e)))?;

    Ok(merchant.into())
}

fn parse_optional_datetime(
    input: Option<String>,
) -> Result<Option<DateTime<Utc>>, ServerFnError> {
    let Some(raw) = input else {
        return Ok(None);
    };
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    let mut normalized = trimmed.replace('T', " ");
    if normalized.len() == 16 {
        normalized.push_str(":00");
    }
    let naive = NaiveDateTime::parse_from_str(&normalized, "%Y-%m-%d %H:%M:%S")
        .map_err(|e| ServerFnError::new(format!("时间格式错误: {}", e)))?;
    let beijing = FixedOffset::east_opt(8 * 3600)
        .ok_or_else(|| ServerFnError::new("时间格式错误".to_string()))?;
    let beijing_dt = beijing
        .from_local_datetime(&naive)
        .single()
        .ok_or_else(|| ServerFnError::new("时间格式错误".to_string()))?;
    Ok(Some(beijing_dt.with_timezone(&Utc)))
}

fn normalize_optional_string(value: Option<String>) -> Option<String> {
    value.and_then(|value| {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}
