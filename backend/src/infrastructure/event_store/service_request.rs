use disintegrate::serde::json::Json;
use disintegrate_postgres::PgEventStore;

use crate::domain::crm::service_request::ServiceRequestEventEnvelope;
use crate::infrastructure::event_store::{event_store_pool, initialize};

pub type ServiceRequestEventStore =
    PgEventStore<ServiceRequestEventEnvelope, Json<ServiceRequestEventEnvelope>>;

pub async fn event_store() -> Result<ServiceRequestEventStore, String> {
    initialize().await?;
    let pool = event_store_pool().await?;

    Ok(PgEventStore::new_uninitialized(
        pool,
        Json::<ServiceRequestEventEnvelope>::default(),
    ))
}
