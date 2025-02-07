use sea_orm::entity::prelude::*;
use sea_orm::{DeriveRelation, EnumIter};

#[derive(Debug, Clone, DeriveEntityModel)]
#[sea_orm(table_name = "track_artists")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub track_id: u32,
    #[sea_orm(primary_key, auto_increment = false)]
    pub artist_id: u32,
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

impl ActiveModelBehavior for ActiveModel {}