use sea_orm::entity::prelude::*;
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Clone, DeriveEntityModel, Serialize, ToSchema)]
#[schema(title = "Album", as = entity::album::Model)]
#[sea_orm(table_name = "albums")]
pub struct Model {
    
    #[sea_orm(primary_key)]
    #[schema(read_only)]
    pub id: u32,
    
    /// Canonical title of the album
    #[schema(examples("Square One"))]
    pub albumtitle: String,
    
    /// Normalized album title
    #[sea_orm(unique)]
    #[serde(skip_serializing, skip_deserializing)]
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

impl Related<super::artist::Entity> for Entity {
    fn to() -> RelationDef { super::album_artist::Relation::Artist.def() }
    fn via() -> Option<RelationDef> { Some(super::album_artist::Relation::Album.def().rev()) }
}

impl ActiveModelBehavior for ActiveModel {}
