use sea_orm::entity::prelude::*;
use serde::Serialize;

#[derive(Debug, Clone, DeriveEntityModel, Serialize)]
#[sea_orm(table_name = "albums")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: u32,
    pub albumtitle: String,
    #[sea_orm(unique)]
    pub albumtitle_normalized: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::track::Entity")]
    Track,
}

impl Related<super::track::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Track.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
