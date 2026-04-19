pub mod order;
pub mod schedule;
pub mod service_request;

use disintegrate::Event;
use disintegrate::serde::json::Json;
use disintegrate_postgres::{Migrator, PgEventStore};
use serde::{Serialize, de::DeserializeOwned};
use std::env;
use std::future::pending;
use tokio::sync::OnceCell;

use crate::domain::crm::order::OrderEventEnvelope;
use crate::domain::crm::schedule::ScheduleEventEnvelope;
use crate::domain::crm::service_request::ServiceRequestEventEnvelope;

static EVENT_STORE_POOL: OnceCell<sqlx::PgPool> = OnceCell::const_new();
static EVENT_STORE_INIT: OnceCell<()> = OnceCell::const_new();
const PROJECTION_LEADER_LOCK_KEY: i64 = 0x5049_434f_4351_5253;

pub(crate) async fn event_store_pool() -> Result<sqlx::PgPool, String> {
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
        .get_or_try_init(|| async move {
            initialize_registered_event_schemas(pool.clone()).await?;
            initialize_listener_infra(pool.clone()).await?;
            backfill_schedule_event_order_uuid(pool).await?;
            Ok::<(), String>(())
        })
        .await?;

    Ok(())
}

async fn initialize_listener_infra(pool: sqlx::PgPool) -> Result<(), String> {
    let event_store =
        PgEventStore::new_uninitialized(pool, Json::<ServiceRequestEventEnvelope>::default());
    Migrator::new(event_store)
        .init_listener()
        .await
        .map_err(|e| format!("initialize event listener schema error: {}", e))
}

pub async fn hold_projection_leader_lock() -> Result<bool, String> {
    let pool = event_store_pool().await?;
    let mut conn = pool
        .acquire()
        .await
        .map_err(|e| format!("acquire projection leader lock connection error: {}", e))?;

    let acquired: bool = sqlx::query_scalar("SELECT pg_try_advisory_lock($1)")
        .bind(PROJECTION_LEADER_LOCK_KEY)
        .fetch_one(&mut *conn)
        .await
        .map_err(|e| format!("acquire projection leader lock error: {}", e))?;

    if !acquired {
        return Ok(false);
    }

    tokio::spawn(async move {
        let _projection_lock_conn = conn;
        pending::<()>().await;
    });

    Ok(true)
}

async fn initialize_registered_event_schemas(pool: sqlx::PgPool) -> Result<(), String> {
    // Add new event-sourced modules here so their domain-id columns/indexes are registered.
    initialize_event_schema::<ServiceRequestEventEnvelope>(pool.clone(), "service request").await?;
    initialize_event_schema::<OrderEventEnvelope>(pool.clone(), "order").await?;
    initialize_event_schema::<ScheduleEventEnvelope>(pool.clone(), "schedule").await?;
    Ok(())
}

async fn initialize_event_schema<E>(pool: sqlx::PgPool, label: &str) -> Result<(), String>
where
    E: Event + Clone + Send + Sync + Serialize + DeserializeOwned + 'static,
{
    PgEventStore::try_new(pool, Json::<E>::default())
        .await
        .map(|_| ())
        .map_err(|e| format!("initialize {} event store schema error: {}", label, e))
}

async fn backfill_schedule_event_order_uuid(pool: sqlx::PgPool) -> Result<(), String> {
    let has_legacy_order_id = sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS (
            SELECT 1
            FROM information_schema.columns
            WHERE table_name = 'event' AND column_name = 'order_id'
        )
        "#,
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| format!("check legacy event.order_id column error: {}", e))?;

    if !has_legacy_order_id {
        return Ok(());
    }

    sqlx::query(
        r#"
        UPDATE event
        SET order_uuid = order_id
        WHERE order_uuid IS NULL
          AND order_id IS NOT NULL
        "#,
    )
    .execute(&pool)
    .await
    .map_err(|e| format!("backfill schedule event order_uuid error: {}", e))?;

    Ok(())
}
