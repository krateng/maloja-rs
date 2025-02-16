use crate::entity::track::TrackRead;
use std::time::Duration;
use sea_orm::entity::prelude::*;
use sea_orm::prelude::Json;
use serde::Serialize;
use utoipa::ToSchema;
use crate::entity::track::TrackWrite;

#[derive(Debug, Clone, DeriveEntityModel, Serialize)]
#[sea_orm(table_name = "scrobbles")]
pub struct Model {
    
    #[sea_orm(primary_key, auto_increment = false)]
    pub timestamp: i64,

    pub track_id: u32,
    
    /// Raw Json of the Scrobble for later reparsing
    pub raw_scrobble: Json,
    
    /// Identifier of the scrobble source, as reported by the submitter
    pub origin: Option<String>,
    
    /// Duration of the scrobble in seconds
    pub listen_duration: Option<u32>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(belongs_to = "super::track::Entity", from = "Column::TrackId", to = "super::track::Column::Id")]
    Track,
}

impl Related<super::track::Entity> for Entity {
    fn to() -> RelationDef { Relation::Track.def() }
}

impl ActiveModelBehavior for ActiveModel {}

/// Representation of a scrobble with the information that can be supplied from the outside.
/// Used for creating or patching a scrobble
#[derive(Clone, Eq, Hash, PartialEq, Debug, Serialize, ToSchema)]
pub struct ScrobbleWrite {
    #[schema(examples(904098042))]
    pub timestamp: i64,
    pub track: TrackWrite,
    #[schema(examples("navidrome"))]
    pub origin: Option<String>,
    #[schema(examples(174))]
    pub listen_duration: Option<u32>,
}

#[derive(Clone, Eq, Hash, PartialEq, Debug, Serialize, ToSchema)]
#[schema(title = "Scrobble", as = entity::scrobble::ScrobbleRead, description = "Instance of user listening to a track")]
pub struct ScrobbleRead {
    #[schema(examples(904098042))]
    pub timestamp: i64,
    pub track: TrackRead,
}
