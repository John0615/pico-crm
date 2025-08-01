//! `SeaORM` Entity, @generated by sea-orm-codegen 1.1.3

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub uuid: Uuid,
    #[sea_orm(unique)]
    pub user_name: String,
    pub password: String,
    #[sea_orm(unique)]
    pub email: Option<String>,
    #[sea_orm(unique)]
    pub phone_number: Option<String>,
    pub is_admin: Option<bool>,
    pub status: String,
    #[sea_orm(column_type = "Text", nullable)]
    pub avatar_url: Option<String>,
    pub last_login_at: Option<DateTime>,
    pub email_verified_at: Option<DateTime>,
    pub inserted_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
