use sea_orm::entity::prelude::*;
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Clone, DeriveEntityModel, Serialize, ToSchema)]
#[schema(title = "Artist", as = entity::artist::Model)]
#[sea_orm(table_name = "artists")]
pub struct Model {
    
    #[sea_orm(primary_key)]
    #[schema(read_only)]
    pub id: i32,
    
    /// Canonical Artist name
    #[schema(examples("Jennie Kim", "TWICE"))]
    pub name: String,
    
    /// Normalized name for the database
    #[sea_orm(unique)]
    #[serde(skip_serializing, skip_deserializing)]
    pub name_normalized: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl Related<super::track::Entity> for Entity {
    fn to() -> RelationDef { super::track_artist::Relation::Track.def() }
    fn via() -> Option<RelationDef> { Some(super::track_artist::Relation::Artist.def().rev()) }
}

impl Related<super::album::Entity> for Entity {
    fn to() -> RelationDef { super::album_artist::Relation::Album.def() }
    fn via() -> Option<RelationDef> { Some(super::album_artist::Relation::Artist.def().rev()) }
}

impl ActiveModelBehavior for ActiveModel {}
