use std::collections::HashSet;
use chrono::Utc;
use itertools::Itertools;
use crate::{database, AppState};
use crate::entity::artist::ArtistWrite;
use crate::entity::{ArtistRead, ArtistWriteContext, TrackRead};
use crate::entity::scrobble::{Model, ScrobbleWrite};
use crate::entity::track::TrackWrite;
use crate::tests::common;
use crate::timeranges::{ValidTimeRange, ALL_TIME};

type TrackInfo = (Vec<&'static str>, &'static str);

fn create_scrobble_writes(tracks: Vec<TrackInfo>) -> Vec<ScrobbleWrite> {
    let mut ts = Utc::now().timestamp();
    let writes: Vec<ScrobbleWrite> = tracks
        .into_iter()
        .map(|track| {
            ts -= 200;
            ScrobbleWrite {
                timestamp: Some(ts),
                track: TrackWrite {
                    id: None,
                    title: Some(track.1.parse().unwrap()),
                    artists: Some(
                        track
                            .0
                            .into_iter()
                            .map(|artist| ArtistWriteContext {
                                id: None,
                                name: Some(artist.parse().unwrap()),
                                alias: None,
                                primary: true,
                                mbid: None,
                                spotify_id: None,
                            })
                            .collect(),
                    ),
                    track_length: None,
                    album: None,
                    mbid: None,
                    spotify_id: None,
                },
                origin: None,
                listen_duration: None,
            }
        })
        .collect();

    writes
}

fn assert_same_track(read: &TrackRead, compare: &TrackRead) {
    // for better debugging
    assert_eq!(
        (read.artists.clone().into_iter().map(|a| a.name).sorted().join(",") , read.title.clone()),
        (compare.artists.clone().into_iter().map(|a| a.name).sorted().join(",") , compare.title.clone())
    );
    assert_eq!(read.id, compare.id);
}

async fn assert_all_submitted_scrobbles_one_track(writes: Vec<ScrobbleWrite>, state: AppState) {
    let writes_amount = writes.len();
    for write in writes {
        state.db.create_scrobbles(vec![write], false, true).await.unwrap();
    }
    let scrobbles = state.db.scrobbles(
        &ALL_TIME, None, None, None,false
    ).await.unwrap();
    assert_eq!(scrobbles.len(), writes_amount, "The incorrect amount of scrobbles has been generated");
    for scrobble in scrobbles.iter() {
        assert_same_track(&scrobble.track, &scrobbles[0].track);
    }
    //let tracks: HashSet<TrackRead> = scrobbles.into_iter().map(|scrobble| scrobble.track).collect();
    //assert_eq!(tracks.len(), 1);
}


fn ignore_capitalization_data() -> Vec<ScrobbleWrite> {
    let tracks: Vec<TrackInfo> = vec![
        (vec!["Jennie", "Lalisa"], "The Ballad of Ace LeBaron"),
        (vec!["Jennie", "Lalisa"], "The Ballad of Ace Lebaron"),
        (vec!["jennie", "lalisa"], "The Ballad of Ace LeBaron"),
        (vec!["jennie", "lalisa"], "The Ballad of Ace Lebaron"),
    ];
    create_scrobble_writes(tracks)
}

#[tokio::test]
async fn ignore_capitalization_separate() {
    let state = common().await;
    let writes = ignore_capitalization_data();
    assert_all_submitted_scrobbles_one_track(writes, state).await;
}

#[tokio::test]
async fn ignore_capitalization_batch() {
    let state = common().await;
    let writes = ignore_capitalization_data();
    assert_all_submitted_scrobbles_one_track(writes, state).await;
}

fn ignore_artist_order_data() -> Vec<ScrobbleWrite> {
    let tracks: Vec<TrackInfo> = vec![
        (vec!["Howard Shore", "Jeremy Soule", "Tilman Sillescu"], "Party in the PRC"),
        (vec!["Tilman Sillescu", "Howard Shore", "Jeremy Soule"], "Party in the PRC"),
        (vec!["Jeremy Soule", "Tilman Sillescu", "Howard Shore"], "Party in the PRC"),
    ];
    create_scrobble_writes(tracks)
}

#[tokio::test]
async fn ignore_artist_order_separate() {
    let state = common().await;
    let writes = ignore_artist_order_data();
    assert_all_submitted_scrobbles_one_track(writes, state).await;
}

#[tokio::test]
async fn ignore_artist_order_batch() {
    let state = common().await;
    let writes = ignore_artist_order_data();
    assert_all_submitted_scrobbles_one_track(writes, state).await;
}

fn ignore_duplicate_artists_data() -> Vec<ScrobbleWrite> {
    let tracks: Vec<TrackInfo> = vec![
        (vec!["Soyeon", "Jennie", "Dahyun"], "VfB Stuttgart, ein Leben lang!"),
        (vec!["Soyeon", "Jennie", "Dahyun", "Soyeon"], "VfB Stuttgart, ein Leben lang!"),
        (vec!["Soyeon", "Jennie", "Dahyun", "Jennie"], "VfB Stuttgart, ein Leben lang!"),
        (vec!["Soyeon", "Jennie", "Dahyun", "Dahyun"], "VfB Stuttgart, ein Leben lang!"),
        (vec!["Soyeon", "Jennie", "Dahyun", "soyeon", "jennie", "dahyun"], "VfB Stuttgart, ein Leben lang!"),
    ];
    create_scrobble_writes(tracks)
}

#[tokio::test]
async fn ignore_duplicate_artists_separate() {
    let state = common().await;
    let writes = ignore_duplicate_artists_data();
    assert_all_submitted_scrobbles_one_track(writes, state).await;
}

#[tokio::test]
async fn ignore_duplicate_artists_batch() {
    let state = common().await;
    let writes = ignore_duplicate_artists_data();
    assert_all_submitted_scrobbles_one_track(writes, state).await;

}

fn parse_multi_artists_data() -> Vec<ScrobbleWrite> {
    let tracks: Vec<TrackInfo> = vec![
        (vec!["Soyeon", "Yuqi"], "Sapphic"),
        (vec!["Yuqi"], "Sapphic ft. Soyeon"),
        (vec!["Yuqi"], "Sapphic (ft. Soyeon)"),
        (vec!["Yuqi"], "Sapphic feat. Soyeon"),
        (vec!["Yuqi"], "Sapphic (feat. Soyeon)"),
        (vec!["Yuqi"], "Sapphic featuring Soyeon"),
        (vec!["Yuqi"], "Sapphic (featuring Soyeon)"),
        (vec!["Yuqi ft. Soyeon"], "Sapphic"),
        (vec!["Yuqi feat. Soyeon"], "Sapphic"),
        (vec!["Yuqi featuring Soyeon"], "Sapphic"),
        (vec!["Yuqi (ft. Soyeon)"], "Sapphic"),
        (vec!["Yuqi (feat. Soyeon)"], "Sapphic"),
        (vec!["Yuqi (featuring Soyeon)"], "Sapphic"),
        (vec!["Yuqi;Soyeon"], "Sapphic"),
    ];
    create_scrobble_writes(tracks)
}

#[tokio::test]
async fn parse_multi_artists_separate() {
    let state = common().await;
    let writes = parse_multi_artists_data();
    assert_all_submitted_scrobbles_one_track(writes, state).await;
}

#[tokio::test]
async fn parse_multi_artists_batch() {
    let state = common().await;
    let writes = parse_multi_artists_data();
    assert_all_submitted_scrobbles_one_track(writes, state).await;
}

#[tokio::test]
async fn provide_artist_by_id() {
    let state = common().await;
    let writes = create_scrobble_writes(vec![
        (vec!["Jihyo"], "The Elven Priestess"),
    ]);
    state.db.create_scrobbles(writes, false, true).await.unwrap();

    let scrobbles = state.db.scrobbles(
        &ALL_TIME, None, None, None,false
    ).await.unwrap();
    assert_eq!(scrobbles.len(), 1, "The incorrect amount of scrobbles has been generated");
    let jihyo_id = scrobbles.first().unwrap().track.artists.first().unwrap().id;

    let new_scrobble = ScrobbleWrite {
        timestamp: None,
        track: TrackWrite {
            id: None,
            title: Some("Thigh Noon".parse().unwrap()),
            artists: Some(vec![
                ArtistWriteContext {
                    id: Some(jihyo_id),
                    name: None,
                    alias: None,
                    primary: true,
                    mbid: None,
                    spotify_id: None,
                }
            ]),
            track_length: None,
            album: None,
            mbid: None,
            spotify_id: None,
        },
        origin: None,
        listen_duration: None,
    };

    state.db.create_scrobbles(vec![new_scrobble], false, true).await.unwrap();
    let scrobbles = state.db.scrobbles(
        &ALL_TIME, None, None, None,false
    ).await.unwrap();
    assert_eq!(scrobbles.len(), 2, "The incorrect amount of scrobbles has been generated");
    let artist_ids: HashSet<u32> = scrobbles.into_iter().map(|scrobble| scrobble.track).flat_map(|x| x.artists).map(|x| x.id).collect();
    assert_eq!(artist_ids.len(), 1);
}

#[tokio::test]
async fn provide_track_by_id() {
    let state = common().await;
    let writes = create_scrobble_writes(vec![
        (vec!["Jihyo"], "The Elven Priestess"),
    ]);
    state.db.create_scrobbles(writes, false, true).await.unwrap();

    let scrobbles = state.db.scrobbles(
        &ALL_TIME, None, None, None,false
    ).await.unwrap();
    assert_eq!(scrobbles.len(), 1, "The incorrect amount of scrobbles has been generated");
    let song_id = scrobbles.first().unwrap().track.id;

    let new_scrobble = ScrobbleWrite {
        timestamp: None,
        track: TrackWrite {
            id: Some(song_id),
            title: None,
            artists: None,
            track_length: None,
            album: None,
            mbid: None,
            spotify_id: None,
        },
        origin: None,
        listen_duration: None,
    };

    state.db.create_scrobbles(vec![new_scrobble], false, true).await.unwrap();
    let scrobbles = state.db.scrobbles(
        &ALL_TIME, None, None, None,false
    ).await.unwrap();
    assert_eq!(scrobbles.len(), 2, "The incorrect amount of scrobbles has been generated");
    let track_ids: HashSet<u32> = scrobbles.into_iter().map(|scrobble| scrobble.track).map(|x| x.id).collect();
    assert_eq!(track_ids.len(), 1);


}
