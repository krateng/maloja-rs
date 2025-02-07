use sea_orm::entity::prelude::*;
use sea_orm::prelude::Json;
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Clone, DeriveEntityModel, Serialize, ToSchema)]
#[schema(title = "Scrobble", as = entity::scrobble::Model)]
#[sea_orm(table_name = "scrobbles")]
pub struct Model {
    
    #[sea_orm(primary_key, auto_increment = false)]
    pub timestamp: i64,
    
    pub track_id: u32,
    
    /// Raw Json of the Scrobble for later reparsing
    #[sea_orm(default_value = "{}")]
    #[schema(examples("{}"))]
    #[serde(skip_serializing, skip_deserializing)]
    pub rawscrobble: Json,
    
    /// Identifier of the scrobble source, as reported by the submitter
    #[schema(examples("navidrome"))]
    pub origin: String,
    
    /// Duration of the scrobble in seconds
    #[schema(examples(174))]
    pub duration: u32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
