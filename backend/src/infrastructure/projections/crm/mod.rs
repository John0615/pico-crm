pub mod order_projection;
pub mod schedule_projection;
pub mod service_request_projection;

use disintegrate_postgres::{PgEventListenerError, RetryAction};
use sea_orm::DatabaseConnection;
use std::fmt::Debug;
use std::time::Duration;

pub async fn spawn_crm_listeners(read_model_db: DatabaseConnection) -> Result<(), String> {
    service_request_projection::spawn_service_request_listener(read_model_db.clone()).await?;
    order_projection::spawn_order_listener(read_model_db.clone()).await?;
    schedule_projection::spawn_schedule_listener(read_model_db).await?;
    Ok(())
}

pub(crate) fn projection_listener_retry<HE: Debug>(
    label: &str,
    err: PgEventListenerError<HE>,
    attempts: usize,
) -> RetryAction {
    let backoff_ms = (200_u64 * 2_u64.pow(attempts.min(5) as u32)).min(5_000);
    if attempts >= 10 {
        eprintln!(
            "{} projection listener aborted after repeated errors: {:?}",
            label, err
        );
        return RetryAction::Abort;
    }

    eprintln!(
        "{} projection listener retrying after transient error (attempt {}): {:?}",
        label,
        attempts + 1,
        err
    );
    RetryAction::Wait {
        duration: Duration::from_millis(backoff_ms),
    }
}
