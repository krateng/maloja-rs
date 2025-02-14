use sea_orm::entity::prelude::*;
use serde::Serialize;
use utoipa::ToSchema;
use super::artist::{ArtistRead, ArtistWrite};
use super::album::AlbumRead;

#[derive(Debug, Clone, DeriveEntityModel, Serialize, ToSchema)]
#[schema(title = "Track", as = entity::track::Model)]
#[sea_orm(table_name = "tracks")]
pub struct Model {
    
    #[sea_orm(primary_key)]
    #[schema(read_only)]
    pub id: u32,

    /// Canonical track title. Should be unique for the combination of artists
    #[schema(examples("Whistle"))]
    pub title: String,
    
    /// Normalized track title for the database
    #[serde(skip_serializing, skip_deserializing)]
    pub title_normalized: String,
    
    /// Duration of the full track in seconds
    #[schema(examples(195))]
    pub length: Option<u32>,
    
    /// ID of the canonical album release that contains this track
    pub album_id: Option<u32>,

    #[sea_orm(unique)]
    #[schema(examples("1d48f0c7-f65f-4e3d-8b3e-b066531b9a67"))]
    pub mbid: Option<String>,

    #[sea_orm(unique)]
    #[schema(examples("6NEoeBLQbOMw92qMeLfI40"))]
    pub spotify_id: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(belongs_to = "super::album::Entity", from = "Column::AlbumId", to = "super::album::Column::Id")]
    // TODO: disable this weird pascal case conversion
    Album,
    #[sea_orm(has_many = "super::track_artist::Entity")]
    TrackArtist,
}

impl Related<super::album::Entity> for Entity {
    fn to() -> RelationDef { Relation::Album.def() }
}

impl Related<super::artist::Entity> for Entity {
    fn to() -> RelationDef { super::track_artist::Relation::Artist.def() }
    fn via() -> Option<RelationDef> { Some(super::track_artist::Relation::Track.def().rev()) }
}

impl ActiveModelBehavior for ActiveModel {}

/// Representation of a track with the information that can be supplied from the outside.
/// Used for creating or patching a track, or to identify a track within another entity which could
/// exist or should be newly created
#[derive(Clone, Eq, Hash, PartialEq, Debug)]
pub struct TrackWrite {
    pub id: Option<u32>,
    pub title: Option<String>,
    pub primary_artists: Option<Vec<ArtistWrite>>,
    pub secondary_artists: Option<Vec<ArtistWrite>>,
    pub length: Option<u32>,
    pub mbid: Option<String>,
    pub spotify_id: Option<String>,
}

/// Representation of a track as it should be shown to the outside, for example in the API.
pub struct TrackRead {
    pub id: u32,
    pub title: String,
    pub primary_artists: Vec<ArtistRead>,
    pub secondary_artists: Vec<ArtistRead>,
    pub album: Option<AlbumRead>,
    pub length: Option<u32>,
    pub mbid: Option<String>,
    pub spotify_id: Option<String>,
}