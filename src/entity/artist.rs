use sea_orm::ActiveValue::Set;
use sea_orm::entity::prelude::*;
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Clone, DeriveEntityModel, Serialize, ToSchema)]
#[schema(title = "Artist", as = entity::artist::Model)]
#[sea_orm(table_name = "artists")]
pub struct Model {
    
    #[sea_orm(primary_key)]
    #[schema(read_only)]
    pub id: u32,
    
    /// Canonical Artist name
    #[schema(examples("Blackpink"))]
    pub name: String,
    
    /// Normalized name for the database
    #[sea_orm(unique)]
    #[serde(skip_serializing, skip_deserializing)]
    pub name_normalized: String,

    #[sea_orm(unique)]
    #[schema(examples("48646387-1664-4c9a-9139-9bfd091b823c"))]
    pub mbid: Option<String>,

    #[sea_orm(unique)]
    #[schema(examples("41MozSoPIsD1dJM0CLPjZF"))]
    pub spotify_id: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::track_artist::Entity")]
    TrackArtist
}

impl Related<super::track::Entity> for Entity {
    fn to() -> RelationDef { super::track_artist::Relation::Track.def() }
    fn via() -> Option<RelationDef> { Some(super::track_artist::Relation::Artist.def().rev()) }
}

impl Related<super::album::Entity> for Entity {
    fn to() -> RelationDef { super::album_artist::Relation::Album.def() }
    fn via() -> Option<RelationDef> { Some(super::album_artist::Relation::Artist.def().rev()) }
}

impl ActiveModelBehavior for ActiveModel {}


/// Artist info that can be input
#[derive(Clone, Eq, Hash, PartialEq, Debug)]
pub struct ArtistWrite {
    pub id: Option<u32>,
    pub name: Option<String>,
    pub mbid: Option<String>,
    pub spotify_id: Option<String>,
}

pub struct ArtistRead {
    pub id: u32,
    pub name: String,
    pub mbid: Option<String>,
    pub spotify_id: Option<String>,
}


