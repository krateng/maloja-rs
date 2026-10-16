use crate::database::errors::MalojaError;
use crate::database::repository::{MalojaDB};
use crate::entity::scrobble::{
    Column as ScrobbleColumn, Entity as ScrobbleEntity, Model as ScrobbleModel,
    Relation as ScrobbleRelation, ScrobbleRead,
};
use crate::entity::track::{
    Column as TrackColumn, Relation as TrackRelation,
};
use crate::entity::track_artist::Column as TrackArtistColumn;
use crate::timeranges::{ValidTimeRange};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder, QuerySelect, QueryTrait, RelationTrait};
use sea_query::JoinType;
use crate::database::repository::helpers::{ensure_single_filter, SingleFilter};
use crate::database::transform::models_to_reads::scrobble_model_to_read;

impl MalojaDB {
    pub async fn scrobbles(
        &self,
        timerange: &ValidTimeRange,
        artist_id: Option<u32>,
        album_id: Option<u32>,
        track_id: Option<u32>,
        new_to_old: bool,
    ) -> Result<Vec<ScrobbleRead>, MalojaError> {
        let real_filter = ensure_single_filter(artist_id, album_id, track_id)?;

        let (from_ts, to_ts) = timerange.timestamp_boundaries();
        let mut query =
            ScrobbleEntity::find().filter(ScrobbleColumn::Timestamp.between(from_ts, to_ts));

        match real_filter {
            SingleFilter::Artist(artist_id) => {
                query = query
                    .join(JoinType::LeftJoin, ScrobbleRelation::Track.def())
                    .join(JoinType::LeftJoin, TrackRelation::TrackArtist.def())
                    .filter(TrackArtistColumn::ArtistId.eq(artist_id));
            }
            SingleFilter::Album(album_id) => {
                query = query
                    .join(JoinType::LeftJoin, ScrobbleRelation::Track.def())
                    .filter(TrackColumn::AlbumId.eq(album_id));
            }
            SingleFilter::Track(track_id) => {
                query = query.filter(ScrobbleColumn::TrackId.eq(track_id));
            }
            SingleFilter::None => {}
        }

        if new_to_old {
            query = query.order_by_desc(ScrobbleColumn::Timestamp);
        } else {
            query = query.order_by_asc(ScrobbleColumn::Timestamp);
        }

        let result: Vec<ScrobbleModel> = query.all(&self.connection).await?;
        let track_ids = result.iter().map(|s| s.track_id).collect();
        let track_map = self.resolve_track_ids_to_models(track_ids).await?;

        let result: Vec<ScrobbleRead> = result.into_iter().map(|s| {
            let track_id = s.track_id;
            let track_model = track_map.get(&track_id).expect("Scrobble must have track").clone();
            scrobble_model_to_read((s, track_model))
        } ).collect();

        Ok(result)
    }
}



