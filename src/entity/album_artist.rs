use sea_orm::entity::prelude::*;
use sea_orm::{DeriveRelation, EnumIter};

#[derive(Debug, Clone, DeriveEntityModel)]
#[sea_orm(table_name = "album_artists")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub album_id: u32,
    #[sea_orm(primary_key, auto_increment = false)]
    pub artist_id: u32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::album::Entity",
        from = "Column::AlbumId",
        to = "super::album::Column::Id"
    )]
    Album,
    #[sea_orm(
        belongs_to = "super::artist::Entity",
        from = "Column::ArtistId",
        to = "super::artist::Column::Id"
    )]
    Artist,
}

impl Related<super::album::Entity> for Entity {
    fn to() -> RelationDef { Relation::Album.def() }
}

impl Related<super::artist::Entity> for Entity {
    fn to() -> RelationDef { Relation::Artist.def() }
}

impl ActiveModelBehavior for ActiveModel {}