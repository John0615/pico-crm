use disintegrate::serde::json::Json;
use disintegrate_postgres::PgEventStore;

use crate::domain::crm::schedule::ScheduleEventEnvelope;
use crate::infrastructure::event_store::{event_store_pool, initialize};

pub type ScheduleEventStore = PgEventStore<ScheduleEventEnvelope, Json<ScheduleEventEnvelope>>;

pub async fn event_store() -> Result<ScheduleEventStore, String> {
    initialize().await?;
    let pool = event_store_pool().await?;

    Ok(PgEventStore::new_uninitialized(
        pool,
        Json::<ScheduleEventEnvelope>::default(),
    ))
}
