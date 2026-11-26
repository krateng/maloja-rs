use serde::{Deserialize, Serialize};
use serde_inline_default::serde_inline_default;

#[derive(Clone, Eq, Hash, PartialEq, Debug, Serialize, Deserialize)]
#[cfg_attr(
    feature = "utoipa",
    derive(utoipa::ToSchema),
    schema(title = "Scrobble (Post)", as = ScrobbleWrite, description = "Instance of user listening to a track")
)]
pub struct ScrobbleWrite {
    #[cfg_attr(feature = "utoipa", schema(examples(904098042)))]
    pub timestamp: Option<i64>,
    pub track: EmbeddedTrackWrite,
    #[cfg_attr(feature = "utoipa", schema(examples(174)))]
    pub listen_duration: Option<u32>,
}

#[derive(Clone, Eq, Hash, PartialEq, Debug, Serialize, Deserialize)]
#[cfg_attr(
    feature = "utoipa",
    derive(utoipa::ToSchema),
    schema(title = "Track (Post)", as = EmbeddedTrackWrite, description = "Instance of user listening to a track")
)]
pub struct EmbeddedTrackWrite {
    /// Referencing a track by ID will always override all other information
    #[cfg_attr(feature = "utoipa", schema(examples(42), minimum = 1))]
    pub id: Option<u32>,
    #[cfg_attr(feature = "utoipa", schema(examples("Whistle")))]
    pub title: Option<String>,
    pub artists: Option<Vec<EmbeddedContextArtistWrite>>,
    #[cfg_attr(feature = "utoipa", schema(examples(195)))]
    pub track_length: Option<u32>,
    pub album: Option<EmbeddedAlbumWrite>,
    #[cfg_attr(feature = "utoipa", schema(examples("1d48f0c7-f65f-4e3d-8b3e-b066531b9a67")))]
    pub mbid: Option<String>,
    #[cfg_attr(feature = "utoipa", schema(examples("6NEoeBLQbOMw92qMeLfI40")))]
    pub spotify_id: Option<String>,
}

#[derive(Clone, Eq, Hash, PartialEq, Debug, Serialize, Deserialize)]
#[cfg_attr(
    feature = "utoipa",
    derive(utoipa::ToSchema),
    schema(title = "Album (Post)", as = EmbeddedAlbumWrite, description = "Studio album / EP / Single a track canonically belongs to")
)]
pub struct EmbeddedAlbumWrite {
    /// Referencing an album by ID will always override all other information
    #[cfg_attr(feature = "utoipa", schema(examples(42), minimum = 1))]
    pub id: Option<u32>,
    #[cfg_attr(feature = "utoipa", schema(examples("Square One")))]
    pub album_title: Option<String>,
    pub album_artists: Option<Vec<EmbeddedContextArtistWrite>>,
    #[cfg_attr(feature = "utoipa", schema(examples("e05f3677-7708-4776-9159-5201559b62d3")))]
    pub mbid: Option<String>,
    #[cfg_attr(feature = "utoipa", schema(examples("0FOOodYRlj7gzh7q7IjmNZ")))]
    pub spotify_id: Option<String>,
}

#[serde_inline_default]
#[derive(Clone, Eq, Hash, PartialEq, Debug, Serialize, Deserialize)]
#[cfg_attr(
    feature = "utoipa",
    derive(utoipa::ToSchema),
    schema(title = "Album Artist / Track Artist (Post)", as = EmbeddedContextArtistWrite, description = "Performer, composer or otherwise credited entity in relation to a specific album or track")
)]
pub struct EmbeddedContextArtistWrite {
    /// Referencing an artist by ID will always override all other information
    #[cfg_attr(feature = "utoipa", schema(examples(42), minimum = 1))]
    pub id: Option<u32>,
    #[cfg_attr(feature = "utoipa", schema(examples("Tom Holkenborg")))]
    pub name: Option<String>,
    #[cfg_attr(feature = "utoipa", schema(examples("Junkie XL")))]
    pub alias: Option<String>,
    #[serde_inline_default(true)]
    pub primary: bool,
    #[cfg_attr(feature = "utoipa", schema(examples("48646387-1664-4c9a-9139-9bfd091b823c")))]
    pub mbid: Option<String>,
    #[cfg_attr(feature = "utoipa", schema(examples("41MozSoPIsD1dJM0CLPjZF")))]
    pub spotify_id: Option<String>,
}