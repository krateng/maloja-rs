use serde::Serialize;

#[derive(Clone, Eq, Hash, PartialEq, Debug, Serialize)]
#[cfg_attr(
    feature = "utoipa",
    derive(utoipa::ToSchema),
    schema(title = "Scrobble", as = read::Scrobble, description = "Instance of user listening to a track")
)]
pub struct Scrobble {
    #[cfg_attr(feature = "utoipa", schema(examples(904098042)))]
    pub timestamp: i64,
    pub time_local: String, //TODO generate in template instead?
    pub track: Track,
}

#[derive(Clone, Eq, Hash, PartialEq, Debug, Serialize)]
#[cfg_attr(
    feature = "utoipa",
    derive(utoipa::ToSchema),
    schema(title = "Track", as = read::track, description = "Musical work")
)]
pub struct Track {
    #[cfg_attr(feature = "utoipa", schema(examples(42), minimum = 1))]
    pub id: u32,
    #[cfg_attr(feature = "utoipa", schema(examples("Whistle")))]
    pub title: String,
    pub artists: Vec<ArtistContext>,
    pub album: Option<Album>,
    #[cfg_attr(feature = "utoipa", schema(examples(195)))]
    pub track_length: Option<u32>,
}

#[derive(Clone, Eq, Hash, PartialEq, Debug, Serialize)]
#[cfg_attr(
    feature = "utoipa",
    derive(utoipa::ToSchema),
    schema(title = "Album", as = read::album, description = "Studio album / EP / Single a track canonically belongs to")
)]
pub struct Album {
    #[cfg_attr(feature = "utoipa", schema(examples(42), minimum = 1))]
    pub id: u32,
    #[cfg_attr(feature = "utoipa", schema(examples("Square One")))]
    pub album_title: String,
    pub album_artists: Vec<ArtistContext>,
}

#[derive(Clone, Eq, Hash, PartialEq, Debug, Serialize)]
#[cfg_attr(
    feature = "utoipa",
    derive(utoipa::ToSchema),
    schema(title = "Album Artist / Track Artist", as = read::artistcontext, description = "Performer, composer or otherwise credited entity in relation to a specific album or track")
)]
pub struct ArtistContext {
    #[cfg_attr(feature = "utoipa", schema(examples(42), minimum = 1))]
    pub id: u32,
    #[cfg_attr(feature = "utoipa", schema(examples("Tom Holkenborg")))]
    pub name: String,
    #[cfg_attr(feature = "utoipa", schema(examples("Junkie XL")))]
    pub alias: Option<String>,
    pub primary: bool,
}

#[derive(Clone, Eq, Hash, PartialEq, Debug, Serialize)]
#[cfg_attr(
    feature = "utoipa",
    derive(utoipa::ToSchema),
    schema(title = "Artist", as = read::artist, description = "Performer, composer or otherwise credited entity")
)]
pub struct Artist {
    #[cfg_attr(feature = "utoipa", schema(examples(42), minimum = 1))]
    pub id: u32,
    #[cfg_attr(feature = "utoipa", schema(examples("Blackpink")))]
    pub name: String,
}


#[derive(Serialize)]
#[cfg_attr(
    feature = "utoipa",
    derive(utoipa::ToSchema),
)]
pub struct Paginated<T: Clone> {
    #[cfg_attr(feature = "utoipa", schema(inline))]
    pub pagination: PaginationInfo,
    #[cfg_attr(feature = "utoipa", schema(inline))]
    pub result: Vec<T>,
}

#[derive(Serialize, Clone, Debug)]
#[cfg_attr(
    feature = "utoipa",
    derive(utoipa::ToSchema),
)]
pub struct PaginationInfo {
    #[cfg_attr(feature = "utoipa", schema(examples(2), minimum = 1))]
    pub page: u32,
    #[cfg_attr(feature = "utoipa", schema(examples(5), minimum = 1))]
    pub pages: u32,
    #[cfg_attr(feature = "utoipa", schema(examples(50), minimum = 1))]
    pub items_per_page: u32,
    #[cfg_attr(feature = "utoipa", schema(examples(244)))]
    pub items_total: u32,
}

#[derive(Serialize, Clone, Debug)]
#[cfg_attr(
    feature = "utoipa",
    derive(utoipa::ToSchema),
)]
pub struct ChartsEntry<T> {
    #[cfg_attr(feature = "utoipa", schema(examples(3), minimum = 1))]
    pub rank: u32,
    pub entry: T,
    #[cfg_attr(feature = "utoipa", schema(examples(1337)))]
    pub scrobbles: u32,
    //pub seconds: u32
}
#[derive(Serialize, Clone, Debug)]
#[cfg_attr(
    feature = "utoipa",
    derive(utoipa::ToSchema),
)]
pub struct TopEntry<T> {
    #[cfg_attr(feature = "utoipa", schema(examples("2024")))]
    pub time_range: String,
    pub entry: T,
    #[cfg_attr(feature = "utoipa", schema(examples(313)))]
    pub scrobbles: u32,
    //pub seconds: u32
}

#[derive(Serialize, Clone, Debug)]
#[cfg_attr(
    feature = "utoipa",
    derive(utoipa::ToSchema),
)]
pub struct PulseEntry {
    #[cfg_attr(feature = "utoipa", schema(examples("2024")))]
    pub time_range: String,
    #[cfg_attr(feature = "utoipa", schema(examples(313)))]
    pub scrobbles: u32,
}

#[derive(Serialize, Clone, Debug)]
#[cfg_attr(
    feature = "utoipa",
    derive(utoipa::ToSchema),
)]
pub struct PerformanceEntry {
    #[cfg_attr(feature = "utoipa", schema(examples("2024")))]
    pub time_range: String,
    #[cfg_attr(feature = "utoipa", schema(examples(3)))]
    pub rank: u32,
}