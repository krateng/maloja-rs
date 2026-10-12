use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use serde_inline_default::serde_inline_default;
use utoipa::ToSchema;
use super::{
    TrackArtistRelation, AlbumArtistRelation,
    Track, Album,
};

#[derive(Eq, PartialEq, Hash, Debug, Clone, DeriveEntityModel, Serialize)]
#[sea_orm(table_name = "artists")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: u32,

    /// Canonical Artist name
    pub name: String,

    /// Normalized name for the database
    #[sea_orm(unique)]
    pub name_normalized: String,

    #[sea_orm(unique)]
    pub mbid: Option<String>,

    #[sea_orm(unique)]
    pub spotify_id: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::track_artist::Entity")]
    TrackArtist,
}

#[rustfmt::skip]
impl Related<Track> for Entity {
    fn to() -> RelationDef { TrackArtistRelation::Track.def() }
    fn via() -> Option<RelationDef> { Some(TrackArtistRelation::Artist.def().rev()) }
}

#[rustfmt::skip]
impl Related<Album> for Entity {
    fn to() -> RelationDef { AlbumArtistRelation::Album.def() }
    fn via() -> Option<RelationDef> { Some(AlbumArtistRelation::Artist.def().rev()) }
}

impl ActiveModelBehavior for ActiveModel {}

/// Representation of an artist with the information that can be supplied from the outside.
/// Used for creating or patching an artist, or to identify an artist within another entity who could
/// exist or should be newly created
#[derive(Clone, Eq, Hash, PartialEq, Debug, Serialize, Deserialize, ToSchema)]
#[schema(title = "Artist (Post)", as = entity::artist::ArtistWrite, description = "Performer, composer or otherwise credited entity")]
pub struct ArtistWrite {
    #[schema(minimum = 1, examples(42))]
    pub id: Option<u32>,
    #[schema(examples("Blackpink"))]
    pub name: Option<String>,
    #[schema(examples("48646387-1664-4c9a-9139-9bfd091b823c"))]
    pub mbid: Option<String>,
    #[schema(examples("41MozSoPIsD1dJM0CLPjZF"))]
    pub spotify_id: Option<String>,
}

/// Representation of an artist as they should be shown to the outside, for example in the API.
#[derive(Clone, Eq, Hash, PartialEq, Debug, Serialize, ToSchema)]
#[schema(title = "Artist", as = entity::artist::ArtistRead, description = "Performer, composer or otherwise credited entity")]
pub struct ArtistRead {
    #[schema(minimum = 1, examples(42))]
    pub id: u32,
    #[schema(examples("Blackpink"))]
    pub name: String,
}

#[serde_inline_default]
#[derive(Clone, Eq, Hash, PartialEq, Debug, Serialize, Deserialize, ToSchema)]
#[schema(title = "Album Artist / Track Artist (Post)", as = entity::artist::ArtistWriteContext, description = "Performer, composer or otherwise credited entity in relation to a specific album or track")]
pub struct ArtistWriteContext {
    #[schema(minimum = 1, examples(42))]
    pub id: Option<u32>,
    #[schema(examples("Tom Holkenborg"))]
    pub name: Option<String>,
    #[schema(examples("Junkie XL"))]
    pub alias: Option<String>,
    #[serde_inline_default(true)]
    pub primary: bool,
    #[schema(examples("48646387-1664-4c9a-9139-9bfd091b823c"))]
    pub mbid: Option<String>,
    #[schema(examples("41MozSoPIsD1dJM0CLPjZF"))]
    pub spotify_id: Option<String>,
}

/// Representation of an artist as they should be shown to the outside when within the context of a related entity (track or album)
#[derive(Clone, Eq, Hash, PartialEq, Debug, Serialize, ToSchema)]
#[schema(title = "Album Artist / Track Artist", as = entity::artist::ArtistReadContext, description = "Performer, composer or otherwise credited entity in relation to a specific album or track")]
pub struct ArtistReadContext {
    #[schema(minimum = 1, examples(42))]
    pub id: u32,
    #[schema(examples("Tom Holkenborg"))]
    pub name: String,
    #[schema(examples("Junkie XL"))]
    pub alias: Option<String>,
    pub primary: bool,
}

impl From<ArtistWriteContext> for ArtistWrite {
    fn from(value: ArtistWriteContext) -> Self {
        ArtistWrite {
            id: value.id,
            name: value.name,
            mbid: value.mbid,
            spotify_id: value.spotify_id,
        }
    }
}

impl ArtistWriteContext {
    pub(crate) fn into_artist_write(self) -> ArtistWrite {
        let result: ArtistWrite = self.into();
        result
    }
}
