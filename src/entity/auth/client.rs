use sea_orm::{ActiveModelBehavior, DeriveEntityModel};
use sea_orm::entity::prelude::*;
use serde::Serialize;
use crate::entity::Scrobble;

#[derive(Eq, PartialEq, Hash, Debug, Clone, DeriveEntityModel, Serialize)]
#[sea_orm(table_name = "auth_clients")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: u32,

    pub identifier: String,

    pub permissions: Option<String>,

    pub active: bool,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::super::scrobble::Entity")]
    Scrobble,
}

#[rustfmt::skip]
impl Related<Scrobble> for Entity {
    fn to() -> RelationDef { Relation::Scrobble.def() }
}

impl ActiveModelBehavior for ActiveModel {}