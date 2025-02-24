use sea_orm::{ColumnTrait, DbErr, EntityTrait, JoinType, QueryFilter, QueryOrder, QuerySelect, RelationTrait};
use sea_query::Expr;
use crate::database::connect;
use crate::database::errors::MalojaError;
use crate::database::repository::{resolve_artist_ids, resolve_track_ids, resolve_album_ids};
use crate::database::views::ChartsEntry;
use crate::entity;
use crate::entity::{
    album::{Entity as Album, Model as AlbumModel, ActiveModel as AlbumActiveModel, Column as AlbumColumn, AlbumWrite, AlbumRead},
    track::{Entity as Track, Model as TrackModel, ActiveModel as TrackActiveModel, Column as TrackColumn, TrackWrite, TrackRead},
    artist::{Entity as Artist, Model as ArtistModel, ActiveModel as ArtistActiveModel, Column as ArtistColumn, ArtistWrite, ArtistRead},
    scrobble::{Entity as Scrobble, Model as ScrobbleModel, ActiveModel as ScrobbleActiveModel, Column as ScrobbleColumn, ScrobbleWrite},
    track_artist::{Entity as TrackArtist, ActiveModel as TrackArtistActiveModel, Column as TrackArtistColumn},
    album_artist::{Entity as AlbumArtist, ActiveModel as AlbumArtistActiveModel, Column as AlbumArtistColumn},
};
use crate::timeranges::TimeRange;
// Alright for all selections, we dont join with additional information - modularity over query performance for now
// we generate stats over IDs and use batch resolve with id maps



pub async fn charts_tracks(timerange: TimeRange, artist_id: Option<u32>, album_id: Option<u32>) -> Result<Vec<ChartsEntry<TrackRead>>, MalojaError> {
    let db = connect().await?;
    let (from_ts, to_ts) = timerange.timestamp_boundaries();
    let mut query = Track::find()
        .select_only()
        .join(JoinType::LeftJoin, entity::track::Relation::Scrobble.def())
        .column_as(TrackColumn::Id, "track_id")
        .column_as(ScrobbleColumn::Timestamp.count(), "scrobbles")
        .column_as(
            Expr::cust("RANK() OVER (ORDER BY COUNT(scrobbles.timestamp) DESC)"),
            "rank"
        )
        .group_by(TrackColumn::Id);
        
    
    if let Some(artist_id) = artist_id {
        query = query
            .join(JoinType::LeftJoin, entity::track::Relation::TrackArtist.def())
            .filter(TrackArtistColumn::ArtistId.eq(artist_id));
    }
    if let Some(album_id) = album_id {
        query = query
            .filter(TrackColumn::AlbumId.eq(album_id));
    }
    query = query
        .filter(ScrobbleColumn::Timestamp.between(from_ts, to_ts))
        .order_by_desc(ScrobbleColumn::Timestamp.count());
    
    let result: Vec<(u32, u32, u32)> = query.into_tuple().all(&db).await?;

    let id_list = result.iter().map(|(id, scrobbles, rank)| id.to_owned()).collect();
    let id_map = resolve_track_ids(id_list, &db).await;

    let charts: Vec<ChartsEntry<TrackRead>> = result.into_iter().map(|(id, scrobbles, rank)| {
        ChartsEntry {
            rank: rank as usize,
            scrobbles: scrobbles,
            entry: id_map[&id].clone()
        }
    }).collect();

    Ok(charts)
}

pub async fn charts_artists(timerange: TimeRange) -> Result<Vec<ChartsEntry<ArtistRead>>, MalojaError> {
    let db = connect().await?;
    let (from_ts, to_ts) = timerange.timestamp_boundaries();
    let mut query = Artist::find()
        .select_only()
        .join(JoinType::LeftJoin, entity::artist::Relation::TrackArtist.def())
        .join(JoinType::LeftJoin, entity::track_artist::Relation::Track.def())
        .join(JoinType::LeftJoin, entity::track::Relation::Scrobble.def())
        .column_as(ArtistColumn::Id, "artist_id")
        .column_as(ScrobbleColumn::Timestamp.count(), "scrobbles")
        .column_as(
            Expr::cust("RANK() OVER (ORDER BY COUNT(scrobbles.timestamp) DESC)"),
            "rank"
        )
        .filter(ScrobbleColumn::Timestamp.between(from_ts, to_ts))
        .group_by(ArtistColumn::Id)
        .order_by_desc(ScrobbleColumn::Timestamp.count());
    
    let result: Vec<(u32, u32, u32)> = query.into_tuple().all(&db).await?;

    let id_list = result.iter().map(|(id, scrobbles, rank)| id.to_owned()).collect();
    let id_map = resolve_artist_ids(id_list, &db).await;

    let charts: Vec<ChartsEntry<ArtistRead>> = result.into_iter().map(|(id, scrobbles, rank)| {
        ChartsEntry {
            rank: rank as usize,
            scrobbles: scrobbles,
            entry: id_map[&id].clone()
        }
    }).collect();

    Ok(charts)
}

pub async fn charts_albums(timerange: TimeRange, artist_id: Option<u32>) -> Result<Vec<ChartsEntry<AlbumRead>>, MalojaError> {
    let db = connect().await?;
    let (from_ts, to_ts) = timerange.timestamp_boundaries();
    let mut query = Album::find()
        .select_only()
        .join(JoinType::LeftJoin, entity::album::Relation::Track.def())
        .join(JoinType::LeftJoin, entity::track::Relation::Scrobble.def())
        .column_as(AlbumColumn::Id, "album_id")
        .column_as(ScrobbleColumn::Timestamp.count(), "scrobbles")
        .column_as(
            Expr::cust("RANK() OVER (ORDER BY COUNT(scrobbles.timestamp) DESC)"),
            "rank"
        )
        .filter(ScrobbleColumn::Timestamp.between(from_ts, to_ts))
        .group_by(AlbumColumn::Id);
    
    if let Some(artist_id) = artist_id {
        query = query
            .join(JoinType::LeftJoin, entity::album::Relation::AlbumArtist.def())
            .filter(AlbumArtistColumn::ArtistId.eq(artist_id));
    }
        
    query = query
        .order_by_desc(ScrobbleColumn::Timestamp.count());
    
    let result: Vec<(u32, u32, u32)> = query.into_tuple().all(&db).await?;

    let id_list = result.iter().map(|(id, scrobbles, rank)| id.to_owned()).collect();
    let id_map = resolve_album_ids(id_list, &db).await;

    let charts: Vec<ChartsEntry<AlbumRead>> = result.into_iter().map(|(id, scrobbles, rank)| {
        ChartsEntry {
            rank: rank as usize,
            scrobbles: scrobbles,
            entry: id_map[&id].clone()
        }
    }).collect();

    Ok(charts)
}