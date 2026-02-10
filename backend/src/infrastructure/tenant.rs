use sea_orm::{
    ConnectionTrait, DatabaseBackend, DatabaseConnection, DatabaseTransaction, Statement,
    TransactionTrait,
};

#[derive(Debug, Clone)]
pub struct TenantContext {
    pub merchant_id: String,
    pub role: String,
    pub schema_name: String,
}

pub fn schema_name_from_merchant(prefix: &str, merchant_id: &str) -> Result<String, String> {
    if merchant_id.is_empty() {
        return Err("merchant_id is empty".to_string());
    }
    let schema_name = format!("{}{}", prefix, merchant_id);
    if !is_safe_schema_name(&schema_name) {
        return Err("invalid schema name".to_string());
    }
    Ok(schema_name)
}

pub fn is_safe_schema_name(schema_name: &str) -> bool {
    !schema_name.is_empty()
        && schema_name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_')
}

pub async fn set_search_path(
    connection: &DatabaseConnection,
    schema_name: &str,
) -> Result<(), String> {
    if !is_safe_schema_name(schema_name) {
        return Err("invalid schema name".to_string());
    }
    let stmt = Statement::from_sql_and_values(
        DatabaseBackend::Postgres,
        "SELECT set_config('search_path', $1, true)",
        vec![schema_name.to_string().into()],
    );
    connection
        .execute(stmt)
        .await
        .map_err(|e| format!("set search_path error: {}", e))?;
    Ok(())
}

pub async fn with_tenant_txn<T, F>(
    connection: &DatabaseConnection,
    schema_name: &str,
    f: F,
) -> Result<T, String>
where
    F: for<'a> FnOnce(&'a DatabaseTransaction) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, String>> + Send + 'a>>,
{
    if !is_safe_schema_name(schema_name) {
        return Err("invalid schema name".to_string());
    }
    let txn = connection
        .begin()
        .await
        .map_err(|e| format!("begin transaction error: {}", e))?;
    let stmt = Statement::from_sql_and_values(
        DatabaseBackend::Postgres,
        "SELECT set_config('search_path', $1, true)",
        vec![schema_name.to_string().into()],
    );
    txn.execute(stmt)
        .await
        .map_err(|e| format!("set search_path error: {}", e))?;

    let result = f(&txn).await?;
    txn.commit()
        .await
        .map_err(|e| format!("commit transaction error: {}", e))?;
    Ok(result)
}
