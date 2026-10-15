use crate::database::errors::MalojaError;
use crate::database::repository::{MalojaDB};
use crate::entity::album::AlbumRead;
use crate::entity::artist::ArtistRead;
use crate::entity::track::TrackRead;

impl MalojaDB {
    pub async fn artist_info(&self, artist_id: u32) -> Result<ArtistRead, MalojaError> {

        let result = self.resolve_artist_ids(vec![artist_id]).await?;
        match result.into_iter().next() {
            Some(result) => Ok(result.1),
            None => Err(MalojaError::ArtistNotFound { id: artist_id }),
        }
    }

    pub async fn track_info(&self, track_id: u32) -> Result<TrackRead, MalojaError> {

        let result = self.resolve_track_ids(vec![track_id]).await?;
        match result.into_iter().next() {
            Some(result) => Ok(result.1),
            None => Err(MalojaError::TrackNotFound { id: track_id }),
        }
    }

    pub async fn album_info(&self, album_id: u32) -> Result<AlbumRead, MalojaError> {

        let result = self.resolve_album_ids(vec![album_id]).await?;
        match result.into_iter().next() {
            Some(result) => Ok(result.1),
            None => Err(MalojaError::AlbumNotFound { id: album_id }),
        }
    }
}


