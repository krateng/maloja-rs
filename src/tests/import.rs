use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;
use sea_query::Iden;
use utoipa::openapi::path::ParameterIn::Path;
use crate::database;
use crate::database::import::import;
use crate::entity::{AlbumRead, ArtistRead, TrackRead};
use crate::tests::common;
use crate::timeranges::ALL_TIME;

const TEST_FILES_FOLDER: &str = "/testing/data/import";

fn assert_track_title(track: &TrackRead, expected: &str) {
    assert_eq!(track.title, expected);
}
fn assert_track_artists(track: &TrackRead, expected: Vec<&str>) {
    let actual: HashSet<String> = track.artists.clone()
        .into_iter().map(|artist| (artist.id, artist.name))
        .collect::<HashMap<u32, _>>().into_values()
        .collect();
    let should = expected.into_iter().map(|x| x.to_string()).collect();
    assert_eq!(actual, should);
}
fn assert_album_title(track: &TrackRead, expected: Option<&str>) {
    assert_eq!(track.album.clone().map(|x| x.album_title), expected.map(|x| x.to_string()));
}
fn assert_album_artists(track: &TrackRead, expected: Option<Vec<&str>>) {
    match (track.album.clone(), expected) {
        (Some(album), Some(expected)) => {
            let actual: HashSet<String> = album.album_artists
                .into_iter().map(|artist| (artist.id, artist.name))
                .collect::<HashMap<u32, _>>().into_values()
                .collect();
            let should = expected.into_iter().map(|x| x.to_string()).collect();
            assert_eq!(actual, should);
        }
        (None, None) => {
            return;
        }
        _ => panic!(),
    }

}

#[tokio::test]
async fn import_maloja() {
    let state = common().await;

    fs::copy(
        PathBuf::from(TEST_FILES_FOLDER).join("maloja_export.json"),
        state.folders.data.join("import").join("maloja_export.json"),
    ).unwrap();

    import(&state).await.unwrap();

    let scrobbles = state.db.scrobbles(
        &ALL_TIME, None, None, None,false
    ).await.unwrap();

    assert_eq!(scrobbles.len(), 3, "The incorrect amount of scrobbles has been generated");

    assert_eq!(scrobbles[0].timestamp, 1731970471);
    assert_track_title(&scrobbles[0].track, "Soyeon's Day Off");
    assert_track_artists(&scrobbles[0].track, vec!["Soyeon"]);
    assert_album_title(&scrobbles[0].track, Some("The Big Journey"));
    assert_album_artists(&scrobbles[0].track, Some(vec!["(G)I-DLE"]));
    assert_eq!(scrobbles[0].track.track_length, Some(300));

    assert_eq!(scrobbles[1].timestamp, 1731985643);
    assert_track_title(&scrobbles[1].track, "Arrival at the Castle");
    assert_track_artists(&scrobbles[1].track, vec!["(G)I-DLE"]);
    assert_album_title(&scrobbles[0].track, Some("The Big Journey"));
    assert_album_artists(&scrobbles[0].track, Some(vec!["(G)I-DLE"]));

    assert_eq!(scrobbles[2].timestamp, 1731985874);
    assert_track_title(&scrobbles[2].track, "Evening at the Campfire");
    assert_track_artists(&scrobbles[2].track, vec!["Soyeon", "Yuqi"]);
    assert_album_title(&scrobbles[0].track, Some("The Big Journey"));
    assert_album_artists(&scrobbles[0].track, Some(vec!["(G)I-DLE"]));
}