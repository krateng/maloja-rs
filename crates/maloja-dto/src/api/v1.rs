use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::de::IgnoredAny;
use crate::timestamp::MalojaTimestamp;

#[derive(Deserialize, Serialize, Debug, Clone)]
#[cfg_attr(
    feature = "utoipa",
    derive(utoipa::IntoParams),
    into_params(parameter_in = Query)
)]
pub struct SubmitScrobble {
    #[serde(alias = "artist", default)]
    // could be artist from multiple query arguments
    pub artists: Option<Vec<String>>,
    pub title: String,
    pub album: Option<String>,
    #[serde(rename = "albumartists")]
    pub album_artists: Option<Vec<String>>,
    pub duration: Option<u32>,
    pub length: Option<u32>,
    pub time: Option<i64>,
    // in the py server, this was always considered true if sent, even nofix=false
    #[serde(
        default,
        rename = "nofix",
        deserialize_with = "presence",
        serialize_with = "as_presence",
        skip_serializing_if = "is_false"
    )]
    pub no_fix: bool,
}

// responses

#[derive(Serialize)]
#[cfg_attr(
    feature = "utoipa",
    derive(utoipa::ToSchema)
)]
#[serde(untagged)]
pub enum ScrobbleResponse {
    Success(ScrobbleResponseSuccess),
    Error(ScrobbleResponseError)
}

#[derive(Clone, Eq, Hash, PartialEq, Debug, Serialize)]
#[cfg_attr(
    feature = "utoipa",
    derive(utoipa::ToSchema)
)]
pub struct ScrobbleResponseSuccess {
    #[cfg_attr(feature = "utoipa", schema(inline))]
    pub status: ResponseStatusSucceeded,
    #[cfg_attr(feature = "utoipa", schema(inline))]
    pub track: ResponseTrack,
    #[cfg_attr(feature = "utoipa", schema(inline))]
    pub desc: String,
    #[cfg_attr(feature = "utoipa", schema(inline))]
    pub warnings: Option<Vec<ResponseWarning>>
}

#[derive(Clone, Eq, Hash, PartialEq, Debug, Serialize)]
#[cfg_attr(
    feature = "utoipa",
    derive(utoipa::ToSchema)
)]
pub struct ScrobbleResponseError {
    #[cfg_attr(feature = "utoipa", schema(inline))]
    pub status: ResponseStatusFailed,
    #[cfg_attr(feature = "utoipa", schema(inline))]
    pub error: ResponseError,
}

#[derive(Clone, Eq, Hash, PartialEq, Debug, Serialize)]
#[cfg_attr(
    feature = "utoipa",
    derive(utoipa::ToSchema)
)]
pub enum ResponseStatusSucceeded {
    Success,
}
#[derive(Clone, Eq, Hash, PartialEq, Debug, Serialize)]
#[cfg_attr(
    feature = "utoipa",
    derive(utoipa::ToSchema)
)]
pub enum ResponseStatusFailed {
    Failure,
    Error,
}

#[derive(Clone, Eq, Hash, PartialEq, Debug, Serialize)]
#[cfg_attr(
    feature = "utoipa",
    derive(utoipa::ToSchema)
)]
pub struct ResponseWarning {
    #[cfg_attr(feature = "utoipa", schema(inline))]
    #[serde(rename = "type")]
    pub warning_type: ResponseWarningType,
    #[cfg_attr(feature = "utoipa", schema(inline))]
    pub value: Option<ResponseWarningOrErrorValue>,
    #[cfg_attr(feature = "utoipa", schema(inline))]
    pub desc: String,
}

#[derive(Clone, Eq, Hash, PartialEq, Debug, Serialize)]
#[cfg_attr(
    feature = "utoipa",
    derive(utoipa::ToSchema)
)]
pub struct ResponseError {
    #[cfg_attr(feature = "utoipa", schema(inline))]
    #[serde(rename = "type")]
    pub error_type: ResponseErrorType,
    #[cfg_attr(feature = "utoipa", schema(inline))]
    pub value: Option<ResponseWarningOrErrorValue>,
    #[cfg_attr(feature = "utoipa", schema(inline))]
    pub desc: String,
}

#[derive(Clone, Eq, Hash, PartialEq, Debug, Serialize)]
#[cfg_attr(
    feature = "utoipa",
    derive(utoipa::ToSchema)
)]
#[serde(untagged)]
pub enum ResponseWarningOrErrorValue {
    Single(String),
    Multiple(Vec<String>),
}

#[derive(Clone, Eq, Hash, PartialEq, Debug, Serialize)]
#[cfg_attr(
    feature = "utoipa",
    derive(utoipa::ToSchema)
)]
#[serde(rename_all = "snake_case")]
pub enum ResponseErrorType {
    UnknownError,
    MalformedB64,
    DuplicateTimestamp,
    EntityDoesNotExist,
    ServerNotReady,
    EntityExists,
    MissingEntityParameter,
    MissingScrobbleData,
    // only part of rust server compat mode
    UnknownPleaseUpgrade,
}

#[derive(Clone, Eq, Hash, PartialEq, Debug, Serialize)]
#[cfg_attr(
    feature = "utoipa",
    derive(utoipa::ToSchema)
)]
#[serde(rename_all = "snake_case")]
pub enum ResponseWarningType {
    ScrobbleExists,
    InvalidKeywordIgnored,
    MixedSchema,
}

#[derive(Clone, Eq, Hash, PartialEq, Debug, Serialize)]
#[cfg_attr(
    feature = "utoipa",
    derive(utoipa::ToSchema)
)]
pub struct ResponseTrack {
    #[cfg_attr(feature = "utoipa", schema(inline))]
    pub artists: Vec<String>,
    #[cfg_attr(feature = "utoipa", schema(inline))]
    pub title: String,
}

fn presence<'de, D: Deserializer<'de>>(d: D) -> Result<bool, D::Error> {
    IgnoredAny::deserialize(d)?;
    Ok(true)
}

fn as_presence<S: Serializer>(_: &bool, s: S) -> Result<S::Ok, S::Error> {
    s.serialize_str("true")
}

fn is_false(b: &bool) -> bool { !*b }