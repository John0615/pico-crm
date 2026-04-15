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
    projections::spawn_all_listeners(read_model_db).await?;
    Ok(())
}
