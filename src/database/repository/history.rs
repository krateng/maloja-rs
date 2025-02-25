use crate::database::connect;
use crate::database::errors::MalojaError;
use crate::database::repository::{charts_albums, charts_artists, charts_tracks, scrobbles};
use crate::database::views::{PerformanceEntry, PulseEntry};
use crate::entity::scrobble::ScrobbleRead;
use crate::timeranges::{RangeType, TimeRange};

/// This is for statistics that represent a development over multiple time ranges

pub async fn pulse(sub_ranges: Vec<TimeRange>, artist_id: Option<u32>, album_id: Option<u32>, track_id: Option<u32>) -> Result<Vec<PulseEntry>, MalojaError> {
    let db = connect().await?;
    let mut result = vec![];
    for subrange in sub_ranges {
        let scrobbles = scrobbles(subrange.clone(), artist_id, album_id, track_id, false).await?.len();
        result.push(PulseEntry {
            time_range: subrange,
            scrobbles: scrobbles as u32,
        })
    }

    Ok(result)
}

pub async fn performance(sub_ranges: Vec<TimeRange>, artist_id: Option<u32>, album_id: Option<u32>, track_id: Option<u32>) -> Result<Vec<PerformanceEntry>, MalojaError> {
    let db = connect().await?;
    let mut result = vec![];
    for subrange in sub_ranges {
        if let Some(artist_id) = artist_id {
            let charts = charts_artists(subrange.clone()).await?; //TODO save DB calls, we only need ID here
            let rank = charts.iter().find(|x| { x.entry.id == artist_id }).map(|x| x.rank).unwrap_or(0);
            result.push(PerformanceEntry {
                time_range: subrange,
                rank: rank as u32,
            })
        }
        else if let Some(album_id) = album_id {
            let charts = charts_albums(subrange.clone(), None).await?; //TODO save DB calls, we only need ID here
            let rank = charts.iter().find(|x| { x.entry.id == album_id }).map(|x| x.rank).unwrap_or(0);
            result.push(PerformanceEntry {
                time_range: subrange,
                rank: rank as u32,
            })
        }
        else if let Some(track_id) = track_id {
            let charts = charts_tracks(subrange.clone(), None, None).await?; //TODO save DB calls, we only need ID here
            let rank = charts.iter().find(|x| { x.entry.id == track_id }).map(|x| x.rank).unwrap_or(0);
            result.push(PerformanceEntry {
                time_range: subrange,
                rank: rank as u32,
            })
        }

    }

    Ok(result)
}