use disintegrate::serde::json::Json;
use disintegrate_postgres::PgEventStore;
use std::env;
use tokio::sync::OnceCell;

use crate::domain::crm::service_request::ServiceRequestEventEnvelope;

pub type ServiceRequestEventStore =
    PgEventStore<ServiceRequestEventEnvelope, Json<ServiceRequestEventEnvelope>>;

static EVENT_STORE_POOL: OnceCell<sqlx::PgPool> = OnceCell::const_new();
static EVENT_STORE_INIT: OnceCell<()> = OnceCell::const_new();

async fn event_store_pool() -> Result<sqlx::PgPool, String> {
    EVENT_STORE_POOL
        .get_or_try_init(|| async {
            let database_url = env::var("ES_DATABASE_URL")
                .map_err(|e| format!("load ES_DATABASE_URL error: {}", e))?;
            sqlx::PgPool::connect(&database_url)
                .await
                .map_err(|e| format!("connect event store sqlx pool error: {}", e))
        })
        .await
        .cloned()
}

pub async fn initialize() -> Result<(), String> {
    let pool = event_store_pool().await?;

    EVENT_STORE_INIT
        .get_or_try_init(|| async {
            PgEventStore::try_new(pool, Json::<ServiceRequestEventEnvelope>::default())
                .await
                .map(|_| ())
                .map_err(|e| format!("initialize service request event store error: {}", e))
        })
        .await?;

    Ok(())
}

pub async fn event_store() -> Result<ServiceRequestEventStore, String> {
    initialize().await?;
    let pool = event_store_pool().await?;

    Ok(PgEventStore::new_uninitialized(
        pool,
        Json::<ServiceRequestEventEnvelope>::default(),
    ))
}
