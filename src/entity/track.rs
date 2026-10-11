use super::{
    AlbumRead, AlbumWrite,
    ArtistReadContext, ArtistWriteContext,
    Album, Scrobble, Artist, TrackArtist,
    TrackArtistRelation
};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Eq, PartialEq, Hash, Debug, Clone, DeriveEntityModel, Serialize)]
#[sea_orm(table_name = "tracks")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: u32,

    /// Canonical track title. Should be unique for the combination of artists
    pub title: String,

    /// Normalized track title for the database
    #[serde(skip_serializing, skip_deserializing)]
    pub title_normalized: String,

    /// Duration of the full track in seconds
    pub track_length: Option<u32>,

    /// ID of the canonical album release that contains this track
    pub album_id: Option<u32>,

    #[sea_orm(unique)]
    pub mbid: Option<String>,

    #[sea_orm(unique)]
    pub spotify_id: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::scrobble::Entity")]
    Scrobble,
    #[sea_orm(
        belongs_to = "super::album::Entity",
        from = "Column::AlbumId",
        to = "super::album::Column::Id"
    )]
    // TODO: disable this weird pascal case conversion
    Album,
    #[sea_orm(has_many = "super::track_artist::Entity")]
    TrackArtist,
}

#[rustfmt::skip]
impl Related<Album> for Entity {
    fn to() -> RelationDef { Relation::Album.def() }
}

#[rustfmt::skip]
impl Related<Scrobble> for Entity {
    fn to() -> RelationDef { Relation::Scrobble.def() }
}

#[rustfmt::skip]
impl Related<Artist> for Entity {
    fn to() -> RelationDef { TrackArtistRelation::Artist.def() }
    fn via() -> Option<RelationDef> { Some(TrackArtistRelation::Track.def().rev()) }
}

// extra relation to the n-to-m-connector-entity since that now contains data
#[rustfmt::skip]
impl Related<TrackArtist> for Entity {
    fn to() -> RelationDef { Relation::TrackArtist.def() }
}

impl ActiveModelBehavior for ActiveModel {}

/// Representation of a track with the information that can be supplied from the outside.
/// Used for creating or patching a track, or to identify a track within another entity which could
/// exist or should be newly created
#[derive(Clone, Eq, Hash, PartialEq, Debug, Serialize, Deserialize, ToSchema)]
#[schema(title = "Track (Post)", as = entity::track::TrackWrite, description = "Musical work")]
pub struct TrackWrite {
    #[schema(minimum = 1, examples(42))]
    pub id: Option<u32>,
    #[schema(examples("Whistle"))]
    pub title: Option<String>,
    pub artists: Option<Vec<ArtistWriteContext>>,
    #[schema(examples(195))]
    pub track_length: Option<u32>,
    pub album: Option<AlbumWrite>,
    #[schema(examples("1d48f0c7-f65f-4e3d-8b3e-b066531b9a67"))]
    pub mbid: Option<String>,
    #[schema(examples("6NEoeBLQbOMw92qMeLfI40"))]
    pub spotify_id: Option<String>,
}

/// Representation of a track as it should be shown to the outside, for example in the API.
#[derive(Clone, Eq, Hash, PartialEq, Debug, Serialize, ToSchema)]
#[schema(title = "Track", as = entity::track::TrackRead, description = "Musical work")]
pub struct TrackRead {
    #[schema(minimum = 1, examples(42))]
    pub id: u32,
    #[schema(examples("Whistle"))]
    pub title: String,
    pub artists: Vec<ArtistReadContext>,
    pub album: Option<AlbumRead>,
    #[schema(examples(195))]
    pub track_length: Option<u32>,
}
