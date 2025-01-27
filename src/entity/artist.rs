use sea_orm::entity::prelude::*;
use serde::Serialize;

#[derive(Debug, Clone, DeriveEntityModel, Serialize)]
#[sea_orm(table_name = "artists")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
    #[sea_orm(unique)]
    pub name_normalized: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
