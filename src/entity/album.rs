use sea_orm::entity::prelude::*;
use serde::Serialize;
use utoipa::ToSchema;
use super::artist::ArtistRead;

#[derive(Debug, Clone, DeriveEntityModel, Serialize, ToSchema)]
#[schema(title = "Album", as = entity::album::Model)]
#[sea_orm(table_name = "albums")]
pub struct Model {
    
    #[sea_orm(primary_key)]
    #[schema(read_only)]
    pub id: u32,
    
    /// Canonical title of the album
    #[schema(examples("Square One"))]
    pub album_title: String,
    
    /// Normalized album title
    #[serde(skip_serializing, skip_deserializing)]
    pub album_title_normalized: String,

    #[sea_orm(unique)]
    #[schema(examples("e05f3677-7708-4776-9159-5201559b62d3"))]
    pub mbid: Option<String>,

    #[sea_orm(unique)]
    #[schema(examples("0FOOodYRlj7gzh7q7IjmNZ"))]
    pub spotify_id: Option<String>,
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

/// Representation of an album with the information that can be supplied from the outside.
/// Used for creating or patching an album, or to identify an album within another entity which could
/// exist or should be newly created
#[derive(Clone, Eq, Hash, PartialEq, Debug)]
pub struct AlbumWrite {
    pub id: Option<u32>,
    pub album_title: Option<String>,
    pub mbid: Option<String>,
    pub spotify_id: Option<String>,
}

/// Representation of an album as it should be shown to the outside, for example in the API.
pub struct AlbumRead {
    pub id: u32,
    pub album_title: String,
    pub album_artists: Vec<ArtistRead>,
}