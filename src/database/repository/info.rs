use sea_orm::DbErr;
use crate::database::connect;
use crate::database::errors::MalojaError;
use crate::database::repository::{resolve_album_ids, resolve_artist_ids, resolve_track_ids};
use crate::entity::album::AlbumRead;
use crate::entity::artist::ArtistRead;
use crate::entity::track::TrackRead;

pub async fn artist_info(artist_id: u32) -> Result<ArtistRead, MalojaError> {
    let db = connect().await?;
    let result = resolve_artist_ids(vec![artist_id], &db).await;
    match result.into_iter().next() {
        Some(result) => { Ok(result.1) }
        None => { Err(MalojaError::ArtistNotFound { id: artist_id }) }
    }
}

pub async fn track_info(track_id: u32) -> Result<TrackRead, MalojaError> {
    let db = connect().await?;
    let result = resolve_track_ids(vec![track_id], &db).await;
    match result.into_iter().next() {
        Some(result) => { Ok(result.1) }
        None => { Err(MalojaError::TrackNotFound { id: track_id }) }
    }
}

pub async fn album_info(album_id: u32) -> Result<AlbumRead, MalojaError> {
    let db = connect().await?;
    let result = resolve_album_ids(vec![album_id], &db).await;
    match result.into_iter().next() {
        Some(result) => { Ok(result.1) }
        None => { Err(MalojaError::AlbumNotFound { id: album_id }) }
    }
}