use crate::database::connect;
use crate::database::errors::MalojaError;
use crate::database::repository::{charts_albums, charts_artists, charts_tracks, ensure_single_filter, scrobbles, SingleFilter};
use crate::database::views::{PerformanceEntry, PulseEntry};
use crate::timeranges::{ValidTimeRange};

/// This is for statistics that represent a development over multiple time ranges

pub async fn pulse(
    sub_ranges: Vec<ValidTimeRange>,
    artist_id: Option<u32>,
    album_id: Option<u32>,
    track_id: Option<u32>,
) -> Result<Vec<PulseEntry>, MalojaError> {
    let _db = connect().await?;
    let mut result = vec![];
    for subrange in sub_ranges {
        let amount_scrobbles =
            scrobbles(&subrange, artist_id, album_id, track_id, false).await?.len();
        result.push(PulseEntry { time_range: subrange, scrobbles: amount_scrobbles as u32 })
    }

    Ok(result)
}

pub async fn performance(
    sub_ranges: Vec<ValidTimeRange>,
    artist_id: Option<u32>,
    album_id: Option<u32>,
    track_id: Option<u32>,
) -> Result<Vec<PerformanceEntry>, MalojaError> {
    let _db = connect().await?;
    let mut result = vec![];
    let real_filter = ensure_single_filter(artist_id, album_id, track_id)?;

    for subrange in sub_ranges {
        match real_filter {
            SingleFilter::Artist(artist_id) => {
                let charts = charts_artists(&subrange).await?; //TODO save DB calls, we only need ID here
                let rank = charts.iter().find(|x| x.entry.id == artist_id).map(|x| x.rank).unwrap_or(0);
                result.push(PerformanceEntry { time_range: subrange.clone(), rank: rank as u32 })
            }
            SingleFilter::Album(album_id) => {
                let charts = charts_albums(&subrange, None).await?; //TODO save DB calls, we only need ID here
                let rank = charts.iter().find(|x| x.entry.id == album_id).map(|x| x.rank).unwrap_or(0);
                result.push(PerformanceEntry { time_range: subrange.clone(), rank: rank as u32 })
            }
            SingleFilter::Track(track_id) => {
                let charts = charts_tracks(&subrange, None, None).await?; //TODO save DB calls, we only need ID here
                let rank = charts.iter().find(|x| x.entry.id == track_id).map(|x| x.rank).unwrap_or(0);
                result.push(PerformanceEntry { time_range: subrange.clone(), rank: rank as u32 })
            }
            SingleFilter::None => {
                return Err(MalojaError::QueryError { message: "Need to specify an artist, album or track.".parse().unwrap() })
            }
        }
    }

    Ok(result)
}
