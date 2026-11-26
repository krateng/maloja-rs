use serde::{Deserialize, Deserializer};
use parse_display::FromStr;
use serde_with::DeserializeFromStr;

#[derive(Deserialize, Debug, Clone)]
pub struct MalojaExport {
    pub maloja: MalojaExportMeta,
    pub scrobbles: Vec<MalojaExportScrobble>,
}

#[derive(Deserialize, Debug, Clone)]
struct MalojaExportMeta {
    pub export_time: u64,
}

#[derive(Deserialize, Debug, Clone)]
pub struct MalojaExportScrobble {
    pub time: i64,
    pub track: MalojaExportTrack,
    pub duration: Option<u32>,
    pub origin: Option<MalojaScrobbleOrigin>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct MalojaExportTrack {
    pub artists: Vec<String>,
    pub title: String,
    pub album: Option<MalojaExportAlbum>,
    pub length: Option<u32>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct MalojaExportAlbum {
    pub artists: Option<Vec<String>>,
    #[serde(alias = "albumtitle")]
    pub album_title: String,
}

#[derive(FromStr, DeserializeFromStr, Debug, Clone, PartialEq, Eq, Hash)]
#[display(style = "lowercase")]
pub enum MalojaScrobbleOrigin {
    Legacy,
    Unknown,
    #[display("import:{0}")]
    Import(MalojaImportSource),
    #[display("client:{identifier}")]
    AttributedClient { identifier: String},
    #[deprecated]
    Generic,
}

#[derive(FromStr, Debug, Clone, PartialEq, Eq, Hash)]
#[display(style = "lowercase")]
pub enum MalojaImportSource {
    LastFM,
    Spotify,
}
