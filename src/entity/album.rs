use std::collections::{BTreeSet, HashMap, HashSet};
use super::{AlbumArtistRelation, ArtistReadContext, ArtistWriteContext, Track, Artist, AlbumArtist, ArtistWrite};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use crate::database::errors::MalojaError;
use crate::database::normalize;

#[derive(Eq, PartialEq, Hash, Debug, Clone, DeriveEntityModel, Serialize)]
#[sea_orm(table_name = "albums")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: u32,

    /// Canonical title of the album
    pub album_title: String,

    /// Normalized album title
    #[serde(skip_serializing, skip_deserializing)]
    pub album_title_normalized: String,

    #[sea_orm(unique)]
    pub mbid: Option<String>,

    #[sea_orm(unique)]
    pub spotify_id: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::track::Entity")]
    Track,
    #[sea_orm(has_many = "super::album_artist::Entity")]
    AlbumArtist,
}

#[rustfmt::skip]
impl Related<Track> for Entity {
    fn to() -> RelationDef { Relation::Track.def() }
}

#[rustfmt::skip]
impl Related<Artist> for Entity {
    fn to() -> RelationDef { AlbumArtistRelation::Artist.def() }
    fn via() -> Option<RelationDef> { Some(AlbumArtistRelation::Album.def().rev()) }
}

// extra relation to the n-to-m-connector-entity since that now contains data
#[rustfmt::skip]
impl Related<AlbumArtist> for Entity {
    fn to() -> RelationDef { Relation::AlbumArtist.def() }
}

impl ActiveModelBehavior for ActiveModel {}

/// Representation of an album with the information that can be supplied from the outside.
/// Used for creating or patching an album, or to identify an album within another entity which could
/// exist or should be newly created
#[derive(Clone, Eq, Hash, PartialEq, Debug, Serialize, Deserialize, ToSchema)]
#[schema(title = "Album (Post)", as = entity::album::AlbumWrite, description = "Studio album / EP / Single a track canonically belongs to")]
pub struct AlbumWrite {
    #[schema(minimum = 1, examples(42))]
    pub id: Option<u32>,
    #[schema(examples("Square One"))]
    pub album_title: Option<String>,
    pub album_artists: Option<Vec<ArtistWriteContext>>,
    #[schema(examples("e05f3677-7708-4776-9159-5201559b62d3"))]
    pub mbid: Option<String>,
    #[schema(examples("0FOOodYRlj7gzh7q7IjmNZ"))]
    pub spotify_id: Option<String>,
}

/// Representation of an album as it should be shown to the outside, for example in the API.
#[derive(Clone, Eq, Hash, PartialEq, Debug, Serialize, ToSchema)]
#[schema(title = "Album", as = entity::album::AlbumRead, description = "Studio album / EP / Single a track canonically belongs to")]
pub struct AlbumRead {
    #[schema(minimum = 1, examples(42))]
    pub id: u32,
    #[schema(examples("Square One"))]
    pub album_title: String,
    pub album_artists: Vec<ArtistReadContext>,
}

#[derive(PartialEq, Eq, Hash, Clone)]
pub(crate) enum UniqueAlbum {
    ById(u32),
    ByInfo{ normalized_title: String, artist_ids: BTreeSet<u32> },
}

impl AlbumWrite {
    pub(crate) fn unique(&self, map: &HashMap<ArtistWrite, u32>) -> Result<UniqueAlbum, MalojaError> {
        match self.id {
            Some(id) => Ok(UniqueAlbum::ById(id)),
            None => {
                if let Some(title) = self.album_title.as_ref() {
                    let artist_ids: Vec<u32> = self.album_artists.clone().unwrap_or_else(Vec::new).iter().map(
                        |a| map.get(&a.clone().into_artist_write()).ok_or_else(
                            || MalojaError::InternalError { message: "Map provided to resolve Artist IDs did not contain entry".to_string() }
                        ).map(|x| x.to_owned()),
                    ).collect::<Result<_, _>>()?;
                    Ok(UniqueAlbum::ByInfo {
                        normalized_title: normalize(title),
                        artist_ids: artist_ids.into_iter().collect(),
                    })
                }
                else {
                    Err(MalojaError::MissingField {
                        field: "album_title".parse().unwrap()
                    })
                }
            }
        }
    }
}