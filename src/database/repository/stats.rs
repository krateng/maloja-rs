use sea_orm::{ColumnTrait, DbErr, EntityTrait, JoinType, QueryOrder, QuerySelect, RelationTrait};
use crate::database::connect;
use crate::database::repository::resolve::{resolve_artist_ids, resolve_track_ids};
use crate::database::views::ChartsEntry;
use crate::entity;
use crate::entity::{
    album::{Entity as Album, Model as AlbumModel, ActiveModel as AlbumActiveModel, Column as AlbumColumn, AlbumWrite, AlbumRead},
    track::{Entity as Track, Model as TrackModel, ActiveModel as TrackActiveModel, Column as TrackColumn, TrackWrite, TrackRead},
    artist::{Entity as Artist, Model as ArtistModel, ActiveModel as ArtistActiveModel, Column as ArtistColumn, ArtistWrite, ArtistRead},
    scrobble::{Entity as Scrobble, Model as ScrobbleModel, ActiveModel as ScrobbleActiveModel, Column as ScrobbleColumn, ScrobbleWrite},
    track_artist::{Entity as TrackArtist, ActiveModel as TrackArtistActiveModel},
    album_artist::{Entity as AlbumArtist, ActiveModel as AlbumArtistActiveModel},
};


// Alright for all selections, we dont join with additional information - modularity over query performance for now
// we generate stats over IDs and use batch resolve with id maps



pub async fn charts_tracks() -> Result<Vec<ChartsEntry<TrackRead>>, DbErr> {
    let db = connect().await?;
    let result: Vec<(u32, u32)> = Track::find()
        .select_only()
        .join(JoinType::LeftJoin, entity::track::Relation::Scrobble.def())
        .column_as(TrackColumn::Id, "track_id")
        .column_as(ScrobbleColumn::Timestamp.count(), "scrobbles")
        .group_by(TrackColumn::Id)
        .order_by_desc(ScrobbleColumn::Timestamp.count())
        .into_tuple()
        .all(&db).await?;

    let id_list = result.iter().map(|(id, scrobbles)| id.to_owned()).collect();
    let id_map = resolve_track_ids(id_list, &db).await;

    let charts: Vec<ChartsEntry<TrackRead>> = result.into_iter().map(|(id, scrobbles)| {
        ChartsEntry {
            rank: 1,
            scrobbles: scrobbles,
            entry: id_map[&id].clone()
        }
    }).collect();

    Ok(charts)
}

pub async fn charts_artists() -> Result<Vec<ChartsEntry<ArtistRead>>, DbErr> {
    let db = connect().await?;
    let result: Vec<(u32, u32)> = Artist::find()
        .select_only()
        .join(JoinType::LeftJoin, entity::artist::Relation::TrackArtist.def())
        .join(JoinType::LeftJoin, entity::track_artist::Relation::Track.def())
        .join(JoinType::LeftJoin, entity::track::Relation::Scrobble.def())
        .column_as(ArtistColumn::Id, "artist_id")
        .column_as(ScrobbleColumn::Timestamp.count(), "scrobbles")
        .group_by(ArtistColumn::Id)
        .order_by_desc(ScrobbleColumn::Timestamp.count())
        .into_tuple()
        .all(&db).await?;

    let id_list = result.iter().map(|(id, scrobbles)| id.to_owned()).collect();
    let id_map = resolve_artist_ids(id_list, &db).await;

    let charts: Vec<ChartsEntry<ArtistRead>> = result.into_iter().map(|(id, scrobbles)| {
        ChartsEntry {
            rank: 1,
            scrobbles: scrobbles,
            entry: id_map[&id].clone()
        }
    }).collect();

    Ok(charts)
}