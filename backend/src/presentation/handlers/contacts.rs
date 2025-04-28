use crate::infrastructure::db::DatabaseConnection;
use sea_orm::{EntityTrait, ActiveModelTrait, PaginatorTrait, QueryOrder};
use sea_orm::entity::prelude::{Uuid};
use sea_orm::ActiveValue::{Set};
use crate::domain::models::contacts::{Column, Entity, ActiveModel};
use chrono::prelude::{Local, DateTime, NaiveDateTime};
use shared::contact::Contact;

pub async fn fetch_contacts(db: &DatabaseConnection) -> Result<Vec<Contact>, String> {
    let paginator = Entity::find()
        .order_by_desc(Column::InsertedAt)
        .paginate(db, 10); // 每页10条
    // 获取当前页数据
    let contacts = paginator
        .fetch_page(0) // 第一页（页码从0开始）
        .await
        .map_err(|_| "获取数据失败".to_string())?;
    // 获取总数
    let total = paginator
        .num_items()
        .await
        .map_err(|_| "获取总数失败".to_string())?;
    let contacts: Vec<Contact> = contacts.into_iter().map(|contact| {
        Contact {
            contact_uuid: contact.contact_uuid.to_string(),
            user_name: contact.user_name,
            company: contact.company,
            position: contact.position,
            phone_number: contact.phone_number,
            email: contact.email,
            last_contact: contact.last_contact.format("%Y-%m-%d %H:%M:%S").to_string(),
            value_level: contact.value_level,
            status: contact.status,
            inserted_at: contact.inserted_at.format("%Y-%m-%d %H:%M:%S").to_string(),
            updated_at: contact.updated_at.format("%Y-%m-%d %H:%M:%S").to_string(),
        }
    }).collect();
    println!("contacts {:#?} {}", contacts, total);
    Ok(contacts)
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
        value_level: Set(contact.value_level),
        creator_uuid: Set(uuid),
        status: Set(contact.status),
        inserted_at: Set(naive_now),
        updated_at: Set(naive_now),
        ..Default::default()
    };
    let item = item.insert(db).await.map_err(|_| "err".to_string())?;
    println!("item {:#?}", item);
    Ok(())
}
