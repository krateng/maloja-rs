use super::{
  TrackRead, TrackWrite,
  Track, Client
};
use sea_orm::entity::prelude::*;
use sea_orm::prelude::Json;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Eq, PartialEq, Hash, Debug, Clone, DeriveEntityModel, Serialize)]
#[sea_orm(table_name = "music_scrobbles")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub timestamp: i64,

    pub track_id: u32,

    /// Raw Json of the Scrobble for later reparsing
    pub raw_scrobble: Json,

    /// Duration of the scrobble in seconds
    pub listen_duration: Option<u32>,

    pub origin_client_id: Option<u32>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::track::Entity",
        from = "Column::TrackId",
        to = "super::track::Column::Id"
    )]
    Track,
    #[sea_orm(
        belongs_to = "super::auth::client::Entity",
        from = "Column::OriginClientId",
        to = "super::auth::client::Column::Id"
    )]
    OriginClient,
}

#[rustfmt::skip]
impl Related<Track> for Entity {
    fn to() -> RelationDef { Relation::Track.def() }
}

#[rustfmt::skip]
impl Related<Client> for Entity {
    fn to() -> RelationDef { Relation::OriginClient.def() }
}

impl ActiveModelBehavior for ActiveModel {}

/// Representation of a scrobble with the information that can be supplied from the outside.
/// Used for creating or patching a scrobble
#[derive(Clone, Eq, Hash, PartialEq, Debug, Serialize, Deserialize, ToSchema)]
#[schema(title = "Scrobble (Post)", as = entity::scrobble::ScrobbleWrite, description = "Instance of user listening to a track")]
pub struct ScrobbleWrite {
    #[schema(examples(904098042))]
    pub timestamp: Option<i64>,
    pub track: TrackWrite,
    #[schema(examples(174))]
    pub listen_duration: Option<u32>,
}

#[derive(Clone, Eq, Hash, PartialEq, Debug, Serialize, ToSchema)]
#[schema(title = "Scrobble", as = entity::scrobble::ScrobbleRead, description = "Instance of user listening to a track")]
pub struct ScrobbleRead {
    #[schema(examples(904098042))]
    pub timestamp: i64,
    pub time_local: String, //TODO generate in template instead?
    pub track: TrackRead,
}
