use crate::database::repository::{timestamp_to_local, AlbumArtistFull, AlbumModelWithRelated, ScrobbleModelWithRelated, TrackArtistFull, TrackModelWithRelated};
use crate::entity::{
    AlbumRead, ArtistModel, ArtistRead,
    ArtistReadContext,
    ScrobbleRead,
    TrackRead
};

// this would be cooler to do via Into trait implementations, but we need already
// associated info from the DB to do the full conversion


pub fn artist_model_to_read(artist: ArtistModel) -> ArtistRead {
    ArtistRead {
        id: artist.id,
        name: artist.name,
    }
}

pub fn track_artist_model_to_read((track_artist, artist): TrackArtistFull) -> ArtistReadContext {
    assert_eq!(track_artist.artist_id, artist.id);
    ArtistReadContext {
        id: artist.id,
        name: artist.name,
        alias: track_artist.artist_alias,
        primary: track_artist.primary,
    }
}

pub fn album_artist_model_to_read((album_artist, artist): AlbumArtistFull) -> ArtistReadContext {
    assert_eq!(album_artist.artist_id, artist.id);
    ArtistReadContext {
        id: artist.id,
        name: artist.name,
        alias: album_artist.artist_alias,
        primary: album_artist.primary,
    }
}

pub fn track_model_to_read((track, track_artists, album): TrackModelWithRelated) -> TrackRead {
    TrackRead {
        id: track.id,
        title: track.title,
        artists: track_artists.into_iter().map(|x| track_artist_model_to_read(x)).collect(),
        album: album.and_then(|x| Some(album_model_to_read(x)) ),
        track_length: None,
    }

}

pub fn album_model_to_read((album, album_artists): AlbumModelWithRelated) -> AlbumRead {
    AlbumRead {
        id: album.id,
        album_title: album.album_title,
        album_artists: album_artists.into_iter().map(|x| album_artist_model_to_read(x) ).collect(),
    }

}

pub fn scrobble_model_to_read((scrobble, track): ScrobbleModelWithRelated) -> ScrobbleRead {
    ScrobbleRead {
        timestamp: scrobble.timestamp.clone(),
        time_local: timestamp_to_local(scrobble.timestamp),
        track: track_model_to_read(track),
    }
}
