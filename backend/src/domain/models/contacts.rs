use sea_orm::entity::prelude::*;
pub use super::_entities::contacts::{self, Column, ActiveModel, Entity, Model};

#[async_trait::async_trait]
impl ActiveModelBehavior for ActiveModel {
    // extend activemodel below (keep comment for generators)

    async fn before_save<C>(self, _db: &C, insert: bool) -> std::result::Result<Self, DbErr>
    where
        C: ConnectionTrait,
    {
        if !insert && self.updated_at.is_unchanged() {
            let mut this = self;
            this.updated_at = sea_orm::ActiveValue::Set(chrono::Local::now().naive_local());
            Ok(this)
        } else {
            Ok(self)
        }
    }
}
