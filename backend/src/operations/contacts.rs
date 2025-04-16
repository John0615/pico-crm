use crate::db::DatabaseConnection;
use sea_orm::EntityTrait;
use crate::models::contacts::Entity;

pub async fn fetch_contacts(db: &DatabaseConnection) -> Result<String, String> {
    let contacts = Entity::find().all(db).await.map_err(|_| "err".to_string())?;

    println!("contacts {:#?}", contacts);
    Ok("abc".to_string())
}
