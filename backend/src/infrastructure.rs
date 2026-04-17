pub mod auth;
pub mod config;
pub mod db;
pub mod entity;
pub mod event_store;
pub mod gateways;
pub mod mappers;
pub mod projections;
pub mod queries;
pub mod repositories;
pub mod tenant;
pub mod utils;

use sea_orm::DatabaseConnection;

pub async fn bootstrap_cqrs(read_model_db: DatabaseConnection) -> Result<(), String> {
    event_store::initialize().await?;
    if !event_store::hold_projection_leader_lock().await? {
        eprintln!(
            "projection leader lock is already held by another process; skipping listener startup"
        );
        return Ok(());
    }
    projections::spawn_all_listeners(read_model_db).await?;
    Ok(())
}
