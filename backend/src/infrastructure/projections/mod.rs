pub mod crm;

use sea_orm::DatabaseConnection;

pub async fn spawn_all_listeners(read_model_db: DatabaseConnection) -> Result<(), String> {
    crm::spawn_crm_listeners(read_model_db).await
}
