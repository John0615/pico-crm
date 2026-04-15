use disintegrate::serde::json::Json;
use disintegrate_postgres::PgEventStore;

use crate::domain::crm::order::OrderEventEnvelope;
use crate::infrastructure::event_store::{event_store_pool, initialize};

pub type OrderEventStore = PgEventStore<OrderEventEnvelope, Json<OrderEventEnvelope>>;

pub async fn event_store() -> Result<OrderEventStore, String> {
    initialize().await?;
    let pool = event_store_pool().await?;

    Ok(PgEventStore::new_uninitialized(
        pool,
        Json::<OrderEventEnvelope>::default(),
    ))
}
