use sea_orm::{DatabaseConnection, DatabaseTransaction, TransactionTrait};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct TenantContext {
    pub merchant_id: String,
    pub role: String,
}

pub fn parse_merchant_uuid(merchant_id: &str) -> Result<Uuid, String> {
    Uuid::parse_str(merchant_id).map_err(|e| format!("invalid merchant uuid: {}", e))
}

pub async fn with_shared_txn<T, F>(connection: &DatabaseConnection, f: F) -> Result<T, String>
where
    F: for<'a> FnOnce(
        &'a DatabaseTransaction,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<T, String>> + Send + 'a>,
    >,
{
    let txn = connection
        .begin()
        .await
        .map_err(|e| format!("begin transaction error: {}", e))?;
    let result = f(&txn).await?;
    txn.commit()
        .await
        .map_err(|e| format!("commit transaction error: {}", e))?;
    Ok(result)
}
