use crate::db::DatabaseConnection;
use sea_orm::{EntityTrait, ActiveModelTrait};
use sea_orm::entity::prelude::{Uuid};
use sea_orm::ActiveValue::{Set, NotSet};
use crate::models::contacts::{Entity, ActiveModel};
use chrono::prelude::{Local, DateTime, NaiveDateTime};
use shared::contact::Contact;

pub async fn fetch_contacts(db: &DatabaseConnection) -> Result<Vec<Contact>, String> {
    let contacts = Entity::find().all(db).await.map_err(|_| "err".to_string())?;

    println!("contacts {:#?}", contacts);
    Ok(vec![])
}

pub async fn create_contact(db: &DatabaseConnection, contact: Contact) -> Result<(), String> {
    println!("contact1111 {:#?}", contact);
    let uuid = Uuid::new_v4();
    let now: DateTime<Local> = Local::now();
    let naive_now: NaiveDateTime = now.naive_local();

    let item = ActiveModel {
        contact_uuid: Set(uuid),
        user_name: Set(contact.user_name),
        company: Set(contact.company),
        position: Set(contact.position),
        phone_number: Set(contact.phone_number),
        email: Set(contact.email),
        last_contact: Set(naive_now),
        value_level: Set(1),
        creator_uuid: Set(uuid),
        status: Set(1),
        inserted_at: Set(naive_now),
        updated_at: Set(naive_now),
        ..Default::default()
    };
    let item = item.insert(db).await.map_err(|_| "err".to_string())?;
    println!("item {:#?}", item);
    Ok(())
}
