use sea_orm::{ActiveModelBehavior, DeriveEntityModel};
use sea_orm::entity::prelude::*;
use serde::Serialize;
use crate::entity::Scrobble;

#[derive(Eq, PartialEq, Hash, Debug, Clone, DeriveEntityModel)]
#[sea_orm(table_name = "auth_clients")]
/// API key or web session used to submit scrobbles or perform other actions. This is basically
/// meant to model an abstract device 'role' - e.g. the 'navidrome' api key can be rotated, can be
/// moved to a new navidrome installation, but consistently represents 'my navidrome server'. In the
/// same fashion, the 'main_desktop' session can rotate actual session keys, or even move from
/// Chrome to Firefox once the user switches, but consistently represents 'my main desktop browser'.
/// Of course, the user can also decide to keep sessions to distinct actual browser installations
/// if that's meant to be tracked in the data
pub struct Model {
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: u32,

    #[sea_orm(unique)]
    pub identifier: String,
    pub active: bool,
    /// Session Key hash or API Key hash
    #[sea_orm(unique)]
    pub token_hash: String,
    /// Displayed shortened key or User Agent
    pub display: String,

    pub created_at: i64,
    pub last_used_at: Option<i64>,
    pub valid_until: Option<i64>,

    // true if browser session, false if api key
    pub session: bool,

}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::super::scrobble::Entity")]
    Scrobble,
}

#[rustfmt::skip]
impl Related<Scrobble> for Entity {
    fn to() -> RelationDef { Relation::Scrobble.def() }
}

impl ActiveModelBehavior for ActiveModel {}