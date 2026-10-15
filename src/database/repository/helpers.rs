use crate::database::errors::MalojaError;

pub enum SingleFilter {
    Artist(u32),
    Album(u32),
    Track(u32),
    None
}

pub fn ensure_single_filter(artist_id: Option<u32>, album_id: Option<u32>, track_id: Option<u32>) -> Result<SingleFilter, MalojaError> {
    match (artist_id, album_id, track_id) {
        (Some(id), None, None) => Ok(SingleFilter::Artist(id)),
        (None, Some(id), None) => Ok(SingleFilter::Album(id)),
        (None, None, Some(id)) => Ok(SingleFilter::Track(id)),
        (None, None, None) => Ok(SingleFilter::None),
        _ => Err(MalojaError::QueryError { message: "Can only filter by either artist, album or track; not multiple.".parse().unwrap() })
    }
}
