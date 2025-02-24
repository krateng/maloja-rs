use crate::database::connect;
use crate::database::errors::MalojaError;
use crate::database::repository::scrobbles;
use crate::database::views::PulseEntry;
use crate::entity::scrobble::ScrobbleRead;
use crate::timeranges::{RangeType, TimeRange};

/// This is for statistics that represent a development over multiple time ranges

pub async fn pulse(sub_ranges: Vec<TimeRange>, artist_id: Option<u32>, album_id: Option<u32>, track_id: Option<u32>) -> Result<Vec<PulseEntry>, MalojaError> {
    let db = connect().await?;
    let mut result = vec![];
    for subrange in sub_ranges {
        let scr = scrobbles(subrange.clone(), artist_id, album_id, track_id, false).await?.len();
        result.push(PulseEntry {
            time_range: subrange,
            scrobbles: scr as u32,
        })
    }
    
    Ok(result)
}