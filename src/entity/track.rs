use sea_orm::entity::prelude::*;
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Clone, DeriveEntityModel, Serialize, ToSchema)]
#[schema(title = "Track", as = entity::track::Model)]
#[sea_orm(table_name = "tracks")]
pub struct Model {
    
    #[sea_orm(primary_key)]
    #[schema(read_only)]
    pub id: u32,

    /// Canonical track title. Should be unique for the combination of artists
    #[schema(examples("As If It's Your Last", "THE THE"))]
    pub title: String,
    
    /// Normalized track title for the database
    #[serde(skip_serializing, skip_deserializing)]
    pub title_normalized: String,
    
    /// Duration of the full track in seconds
    #[schema(examples(195))]
    pub length: u32,
    
    /// ID of the canonical album release that contains this track
    pub album_id: u32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(belongs_to = "super::album::Entity", from = "Column::AlbumId", to = "super::album::Column::Id")]
    // TODO: disable this weird pascal case conversion
    Album,
}

impl Related<super::album::Entity> for Entity {
    fn to() -> RelationDef { Relation::Album.def() }
}

impl Related<super::artist::Entity> for Entity {
    fn to() -> RelationDef { super::track_artist::Relation::Artist.def() }
    fn via() -> Option<RelationDef> { Some(super::track_artist::Relation::Track.def().rev()) }
}

impl ActiveModelBehavior for ActiveModel {}
