use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder, QuerySelect, RelationTrait};
use sea_query::JoinType;
use crate::database::connect;
use crate::database::errors::MalojaError;
use crate::database::repository::resolve_track_ids;
use crate::entity::scrobble::{ScrobbleRead, Entity as ScrobbleEntity, Column as ScrobbleColumn, Relation as ScrobbleRelation, Model as ScrobbleModel};
use crate::entity::track::{TrackRead, Entity as TrackEntity, Column as TrackColumn, Relation as TrackRelation};
use crate::entity::track_artist::{Column as TrackArtistColumn};
use crate::timeranges::TimeRange;

pub async fn scrobbles(timerange: TimeRange, artist_id: Option<u32>, album_id: Option<u32>, track_id: Option<u32>, new_to_old: bool) -> Result<Vec<ScrobbleRead>, MalojaError> {
    assert!(
        (artist_id.is_none() && album_id.is_none()) || (artist_id.is_none() && track_id.is_none()) || (track_id.is_none() && album_id.is_none())
    );
    let db = connect().await?;
    let (from_ts, to_ts) = timerange.timestamp_boundaries();
    let mut query = ScrobbleEntity::find()
        .filter(ScrobbleColumn::Timestamp.between(from_ts, to_ts));
    if let Some(artist_id) = artist_id {
        query = query
            .join(JoinType::LeftJoin, ScrobbleRelation::Track.def())
            .join(JoinType::LeftJoin, TrackRelation::TrackArtist.def())
            .filter(TrackArtistColumn::ArtistId.eq(artist_id));
    };
    if let Some(album_id) = album_id {
        query = query
            .join(JoinType::LeftJoin, ScrobbleRelation::Track.def())
            .filter(TrackColumn::AlbumId.eq(album_id));
    };
    if let Some(track_id) = track_id {
        query = query
            .filter(ScrobbleColumn::TrackId.eq(track_id));
    };

    if new_to_old {
        query = query.order_by_desc(ScrobbleColumn::Timestamp);
    }
    else {
        query = query.order_by_asc(ScrobbleColumn::Timestamp);
    }
    
    let result: Vec<ScrobbleModel> = query.all(&db).await?;
    let track_ids = result.iter().map(|s| s.track_id.clone()).collect();
    let track_map = resolve_track_ids(track_ids, &db).await;


    let result = result.into_iter().map(|s| {
        let tz = chrono_tz::Tz::Europe__Vienna; //TODO
        let time = chrono::DateTime::from_timestamp(s.timestamp.clone(), 0).unwrap();
        let fmt = "%d. %b %Y %H:%M %Z";
        let local_time = time.with_timezone(&tz);
        
        ScrobbleRead {
            timestamp: s.timestamp.clone(),
            time_local: local_time.format(fmt).to_string(),
            track: track_map[&s.track_id].clone(),
        }
    }).collect();

    Ok(result)
}