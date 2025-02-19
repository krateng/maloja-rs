use std::collections::HashMap;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use crate::entity::{
    album::{Entity as Album, Model as AlbumModel, ActiveModel as AlbumActiveModel, Column as AlbumColumn, AlbumWrite, AlbumRead},
    track::{Entity as Track, Model as TrackModel, ActiveModel as TrackActiveModel, Column as TrackColumn, TrackWrite, TrackRead},
    artist::{Entity as Artist, Model as ArtistModel, ActiveModel as ArtistActiveModel, Column as ArtistColumn, ArtistWrite, ArtistRead, ArtistReadContext},
    scrobble::{Entity as Scrobble, Model as ScrobbleModel, ActiveModel as ScrobbleActiveModel, Column as ScrobbleColumn, ScrobbleWrite},
    track_artist::{Entity as TrackArtist, ActiveModel as TrackArtistActiveModel},
    album_artist::{Entity as AlbumArtist, ActiveModel as AlbumArtistActiveModel},
};


pub async fn resolve_track_ids(ids: Vec<u32>, db: &DatabaseConnection) -> HashMap<u32,TrackRead> {
    // we can resolve one relation directly with a db call instead of the function
    let db_result = Track::find()
        .filter(TrackColumn::Id.is_in(ids))
        .find_with_related(Artist)
        .all(db).await.unwrap();

    let album_ids: Vec<u32> = db_result.iter().filter_map(|(track, _)| track.album_id).collect();
    let album_map = resolve_album_ids(album_ids, db).await;

    let mut result: HashMap<u32, TrackRead> = HashMap::new();

    for (track, artists) in db_result {
        result.insert(track.id, TrackRead {
            id: track.id,
            title: track.title,
            artists: artists.into_iter().map(|a| {
                ArtistReadContext {
                    id: a.id,
                    name: a.name,
                    alias: None,
                    primary: true
                }
            }).collect(),
            album: track.album_id.map(|album_id| album_map[&album_id].clone()),
            track_length: track.track_length,
        });
    }

    result

}

pub async fn resolve_album_ids(ids: Vec<u32>, db: &DatabaseConnection) -> HashMap<u32, AlbumRead> {
    let db_result = Album::find()
        .filter(AlbumColumn::Id.is_in(ids))
        .find_with_related(Artist)
        .all(db).await.unwrap();

    let mut result: HashMap<u32, AlbumRead> = HashMap::new();

    for (album, artists) in db_result {
        result.insert(album.id, AlbumRead {
            id: album.id,
            album_title: album.album_title,
            album_artists: artists.into_iter().map(|a| {
                ArtistReadContext {
                    id: a.id,
                    name: a.name,
                    alias: None,
                    primary: true,
                }
            }).collect(),
        });
    }

    result

}

pub async fn resolve_artist_ids(ids: Vec<u32>, db: &DatabaseConnection) -> HashMap<u32, ArtistRead> {
    let db_result = Artist::find()
        .filter(ArtistColumn::Id.is_in(ids))
        .all(db).await.unwrap();

    let mut result: HashMap<u32, ArtistRead> = HashMap::new();

    for artist in db_result {
        result.insert(artist.id, ArtistRead {
            id: artist.id,
            name: artist.name,
        });
    }

    result
}