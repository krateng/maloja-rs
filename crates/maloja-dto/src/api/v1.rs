use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::de::IgnoredAny;
use crate::timestamp::MalojaTimestamp;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SubmitScrobble {
    #[serde(alias = "artist")]
    // could be artist from multiple query arguments
    artists: Option<Vec<String>>,
    title: String,
    album: Option<String>,
    #[serde(rename = "albumartists")]
    album_artists: Option<Vec<String>>,
    duration: Option<u32>,
    length: Option<u32>,
    time: MalojaTimestamp,
    // in the py server, this was always considered true if sent, even nofix=false
    #[serde(
        default,
        rename = "nofix",
        deserialize_with = "presence",
        serialize_with = "as_presence",
        skip_serializing_if = "is_false"
    )]
    no_fix: bool,
}

fn presence<'de, D: Deserializer<'de>>(d: D) -> Result<bool, D::Error> {
    IgnoredAny::deserialize(d)?;
    Ok(true)
}

fn as_presence<S: Serializer>(_: &bool, s: S) -> Result<S::Ok, S::Error> {
    s.serialize_str("true")
}

fn is_false(b: &bool) -> bool { !*b }