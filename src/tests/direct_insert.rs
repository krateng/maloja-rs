use crate::database::repository::{
    get_or_create_albums, get_or_create_artists, get_or_create_tracks,
};
use crate::entity::album::AlbumWrite;
use crate::entity::artist::ArtistWrite;
use crate::entity::track::TrackWrite;
use crate::tests::common;

#[tokio::test]
async fn insert_artists() {
    common().await;
    let write = ArtistWrite {
        id: None,
        name: Some("(G)I-DLE".parse().unwrap()),
        mbid: None,
        spotify_id: None,
    };
    let result = get_or_create_artists(vec![write.clone()]).await.unwrap();

    assert_eq!(result.get(&write).expect("").name, "(G)I-DLE");
}

#[tokio::test]
async fn insert_albums() {
    common().await;
    let write = AlbumWrite {
        id: None,
        album_title: Some("Square One".parse().unwrap()),
        album_artists: Some(vec![ArtistWrite {
            id: None,
            name: Some("Blackpink".parse().unwrap()),
            mbid: None,
            spotify_id: None,
        }]),
        mbid: None,
        spotify_id: None,
    };
    let result = get_or_create_albums(vec![write.clone()]).await.unwrap();
    let result_model = result.get(&write).expect("");
    assert_eq!(result_model.album_title, "Square One");
}

#[tokio::test]
async fn insert_tracks() {
    common().await;
    let write = TrackWrite {
        id: None,
        title: Some("Red Dress".parse().unwrap()),
        primary_artists: Some(vec![ArtistWrite {
            id: None,
            name: Some("Red Velvet".parse().unwrap()),
            mbid: None,
            spotify_id: None,
        }]),
        secondary_artists: None,
        track_length: None,
        album: Some(AlbumWrite {
            id: None,
            album_title: Some("The Red".parse().unwrap()),
            album_artists: Some(vec![ArtistWrite {
                id: None,
                name: Some("Red Velvet".parse().unwrap()),
                mbid: None,
                spotify_id: None,
            }]),
            mbid: None,
            spotify_id: None,
        }),
        mbid: None,
        spotify_id: None,
    };
    let result = get_or_create_tracks(vec![write.clone()]).await.unwrap();

    assert_eq!(result.get(&write).expect("").title, "Red Dress");
}
