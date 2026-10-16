use std::time::{SystemTime, UNIX_EPOCH};
use sea_orm::{NotSet, Set};
use crate::database::normalize;
use crate::entity::{AlbumActiveModel, AlbumRead, AlbumWrite, ArtistActiveModel, ArtistModel, ArtistRead, ArtistReadContext, ArtistWrite, ScrobbleActiveModel, ScrobbleRead, ScrobbleWrite, TrackActiveModel, TrackRead, TrackWrite};

// these for now assume POST. we deal with PATCH operations later
// this means none can just become Set(none), no further logic needed
pub fn artist_write_to_model(artist: ArtistWrite) -> ArtistActiveModel {
    ArtistActiveModel {
        id: NotSet,
        name: Set(artist.name.clone().expect("Cannot save artist without name.")),
        name_normalized: Set(normalize(&*artist.name.expect("Cannot save artist without name."))),
        mbid: Set(artist.mbid),
        spotify_id: Set(artist.spotify_id),
    }
}

pub fn track_write_to_model(track: TrackWrite) -> TrackActiveModel {
    TrackActiveModel {
        id: NotSet,
        title: Set(track.title.clone().expect("Cannot save track without title.")),
        title_normalized: Set(normalize(&*track.title.expect("Cannot save track without title."))),
        track_length: Set(track.track_length),
        album_id: NotSet, // relations are handled outside of this
        mbid: Set(track.mbid),
        spotify_id: Set(track.spotify_id),
    }
}

pub fn album_write_to_model(album: AlbumWrite) -> AlbumActiveModel {
    AlbumActiveModel {
        id: NotSet,
        album_title: Set(album.album_title.clone().expect("Cannot save album without title.")),
        album_title_normalized: Set(normalize(&*album.album_title.expect("Cannot save album without title."))),
        mbid: Set(album.mbid),
        spotify_id: Set(album.spotify_id),
    }
}

pub fn scrobble_write_to_model(scrobble: ScrobbleWrite) -> ScrobbleActiveModel {
    ScrobbleActiveModel {
        timestamp: Set(scrobble.timestamp.unwrap_or_else(|| SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64)),
        track_id: NotSet,
        raw_scrobble: Set("{}".parse().unwrap()), //TODO
        origin: NotSet,
        listen_duration: Set(scrobble.listen_duration),
    }
}