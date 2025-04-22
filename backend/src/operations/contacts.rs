use crate::db::DatabaseConnection;
use sea_orm::{EntityTrait, ActiveModelTrait};
use crate::models::contacts::{Entity, ActiveModel};
use shared::contact::Contact;

pub async fn fetch_contacts(db: &DatabaseConnection) -> Result<Vec<Contact>, String> {
    let contacts = Entity::find().all(db).await.map_err(|_| "err".to_string())?;

    println!("contacts {:#?}", contacts);
    Ok(vec![])
}

pub async fn create_contact(db: &DatabaseConnection, contact: Contact) -> Result<(), String> {
    let item = ActiveModel {
        ..Default::default()
    };
    let item = item.insert(db).await.map_err(|_| "err".to_string())?;
    println!("item {:#?}", item);
    Ok(())
}
