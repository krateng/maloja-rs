use crate::database::errors::MalojaError;
use crate::database::repository::{MalojaDB};
use crate::database::views::ChartsEntry;
use crate::entity;
use crate::entity::{
    album::{
        AlbumRead, Column as AlbumColumn,
        Entity as Album,
    },
    album_artist::Column as AlbumArtistColumn,
    artist::{
        ArtistRead, Column as ArtistColumn,
        Entity as Artist,
    },
    scrobble::Column as ScrobbleColumn,
    track::{
        Column as TrackColumn, Entity as Track
        , TrackRead,
    },
    track_artist::Column as TrackArtistColumn,
};
use crate::timeranges::{ValidTimeRange};
use sea_orm::{
    ColumnTrait, EntityTrait, JoinType, QueryFilter, QueryOrder, QuerySelect, RelationTrait,
};
use sea_query::Expr;
use crate::database::repository::helpers::{ensure_single_filter, SingleFilter};
// Alright for all selections, we dont join with additional information - modularity over query performance for now
// we generate stats over IDs and use batch resolve with id maps

impl MalojaDB {
    pub async fn charts_tracks(
        &self,
        timerange: &ValidTimeRange,
        artist_id: Option<u32>,
        album_id: Option<u32>,
    ) -> Result<Vec<ChartsEntry<TrackRead>>, MalojaError> {
        let real_filter = ensure_single_filter(artist_id, album_id, None)?;

        let (from_ts, to_ts) = timerange.timestamp_boundaries();
        let mut query = Track::find()
            .select_only()
            .join(JoinType::LeftJoin, entity::track::Relation::Scrobble.def())
            .column_as(TrackColumn::Id, "track_id")
            .column_as(ScrobbleColumn::Timestamp.count(), "scrobbles")
            .column_as(Expr::cust("RANK() OVER (ORDER BY COUNT(scrobbles.timestamp) DESC)"), "rank")
            .group_by(TrackColumn::Id);

        match real_filter {
            SingleFilter::Artist(artist_id) => {
                query = query
                    .join(JoinType::LeftJoin, entity::track::Relation::TrackArtist.def())
                    .filter(TrackArtistColumn::ArtistId.eq(artist_id));
            }
            SingleFilter::Album(album_id) => {
                query = query.filter(TrackColumn::AlbumId.eq(album_id));
            }
            SingleFilter::None => {}
            _ => unreachable!(),
        }

        query = query
            .filter(ScrobbleColumn::Timestamp.between(from_ts, to_ts))
            .order_by_desc(ScrobbleColumn::Timestamp.count());

        let result: Vec<(u32, u32, u32)> = query.into_tuple().all(&self.connection).await?;

        let id_list = result.iter().map(|(id, _scrobbles, _rank)| id.to_owned()).collect();
        let id_map = self.resolve_track_ids(id_list).await?;

        let charts: Vec<ChartsEntry<TrackRead>> = result
            .into_iter()
            .map(|(id, scrobbles, rank)| ChartsEntry {
                rank: rank as usize,
                scrobbles: scrobbles,
                entry: id_map[&id].clone(),
            })
            .collect();

        Ok(charts)
    }

    pub async fn charts_artists(
        &self,
        timerange: &ValidTimeRange,
    ) -> Result<Vec<ChartsEntry<ArtistRead>>, MalojaError> {
        let (from_ts, to_ts) = timerange.timestamp_boundaries();
        let query = Artist::find()
            .select_only()
            .join(JoinType::LeftJoin, entity::artist::Relation::TrackArtist.def())
            .join(JoinType::LeftJoin, entity::track_artist::Relation::Track.def())
            .join(JoinType::LeftJoin, entity::track::Relation::Scrobble.def())
            .column_as(ArtistColumn::Id, "artist_id")
            .column_as(ScrobbleColumn::Timestamp.count(), "scrobbles")
            .column_as(Expr::cust("RANK() OVER (ORDER BY COUNT(scrobbles.timestamp) DESC)"), "rank")
            .filter(ScrobbleColumn::Timestamp.between(from_ts, to_ts))
            .group_by(ArtistColumn::Id)
            .order_by_desc(ScrobbleColumn::Timestamp.count());

        let result: Vec<(u32, u32, u32)> = query.into_tuple().all(&self.connection).await?;

        let id_list = result.iter().map(|(id, _scrobbles, _rank)| id.to_owned()).collect();
        let id_map = self.resolve_artist_ids(id_list).await?;

        let charts: Vec<ChartsEntry<ArtistRead>> = result
            .into_iter()
            .map(|(id, scrobbles, rank)| ChartsEntry {
                rank: rank as usize,
                scrobbles: scrobbles,
                entry: id_map[&id].clone(),
            })
            .collect();

        Ok(charts)
    }

    pub async fn charts_albums(
        &self,
        timerange: &ValidTimeRange,
        artist_id: Option<u32>,
    ) -> Result<Vec<ChartsEntry<AlbumRead>>, MalojaError> {
        let (from_ts, to_ts) = timerange.timestamp_boundaries();
        let mut query = Album::find()
            .select_only()
            .join(JoinType::LeftJoin, entity::album::Relation::Track.def())
            .join(JoinType::LeftJoin, entity::track::Relation::Scrobble.def())
            .column_as(AlbumColumn::Id, "album_id")
            .column_as(ScrobbleColumn::Timestamp.count(), "scrobbles")
            .column_as(Expr::cust("RANK() OVER (ORDER BY COUNT(scrobbles.timestamp) DESC)"), "rank")
            .filter(ScrobbleColumn::Timestamp.between(from_ts, to_ts))
            .group_by(AlbumColumn::Id);

        if let Some(artist_id) = artist_id {
            query = query
                .join(JoinType::LeftJoin, entity::album::Relation::AlbumArtist.def())
                .filter(AlbumArtistColumn::ArtistId.eq(artist_id));
        }

        query = query.order_by_desc(ScrobbleColumn::Timestamp.count());

        let result: Vec<(u32, u32, u32)> = query.into_tuple().all(&self.connection).await?;

        let id_list = result.iter().map(|(id, _scrobbles, _rank)| id.to_owned()).collect();
        let id_map = self.resolve_album_ids(id_list).await?;

        let charts: Vec<ChartsEntry<AlbumRead>> = result
            .into_iter()
            .map(|(id, scrobbles, rank)| ChartsEntry {
                rank: rank as usize,
                scrobbles: scrobbles,
                entry: id_map[&id].clone(),
            })
            .collect();

        Ok(charts)
    }
}


