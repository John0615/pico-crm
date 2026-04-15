pub mod order_projection;
pub mod schedule_projection;
pub mod service_request_projection;

use sea_orm::DatabaseConnection;

pub async fn spawn_crm_listeners(read_model_db: DatabaseConnection) -> Result<(), String> {
    service_request_projection::spawn_service_request_listener(read_model_db.clone()).await?;
    order_projection::spawn_order_listener(read_model_db.clone()).await?;
    schedule_projection::spawn_schedule_listener(read_model_db).await?;
    Ok(())
}
