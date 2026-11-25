use sea_orm::{ActiveModelBehavior, DeriveEntityModel};
use sea_orm::entity::prelude::*;
use crate::entity::auth::{Singleton, StoredPasswordHash};

#[derive(Eq, PartialEq, Hash, Debug, Clone, DeriveEntityModel)]
#[sea_orm(table_name = "auth_logins")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Singleton,
    pub password_hash: StoredPasswordHash,
    pub app_generated: bool,
}



#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}