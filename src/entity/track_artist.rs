use sea_orm::entity::prelude::*;
use sea_orm::{DeriveRelation, EnumIter};

#[derive(Debug, Clone, DeriveEntityModel)]
#[sea_orm(table_name = "track_artists")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub track_id: u32,
    #[sea_orm(primary_key, auto_increment = false)]
    pub artist_id: u32,
    pub primary: bool,
    pub artist_alias: Option<String>,
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
        belongs_to = "super::artist::Entity",
        from = "Column::ArtistId",
        to = "super::artist::Column::Id"
    )]
    Artist,
}

impl Related<super::track::Entity> for Entity {
    fn to() -> RelationDef { Relation::Track.def() }
}

impl Related<super::artist::Entity> for Entity {
    fn to() -> RelationDef { Relation::Artist.def() }
}

impl ActiveModelBehavior for ActiveModel {}