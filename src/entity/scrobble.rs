use sea_orm::entity::prelude::*;
use sea_orm::prelude::Json;
use serde::Serialize;

#[derive(Debug, Clone, DeriveEntityModel, Serialize)]
#[sea_orm(table_name = "scrobbles")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub timestamp: i64,
    #[sea_orm(default_value = "{}")]
    pub rawscrobble: Json,
    pub origin: String,
    pub duration: u32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
