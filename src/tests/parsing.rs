use crate::database;
use crate::entity::artist::ArtistWrite;
use crate::entity::ArtistWriteContext;
use crate::entity::scrobble::{Model, ScrobbleWrite};
use crate::entity::track::TrackWrite;
use crate::tests::common;

type TrackInfo = (Vec<&'static str>, &'static str);

fn create_scrobble_writes(tracks: Vec<TrackInfo>) -> Vec<ScrobbleWrite> {
    let mut ts = 100000;
    let writes: Vec<ScrobbleWrite> = tracks
        .into_iter()
        .map(|track| {
            ts += 200;
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

fn assert_all_same_track(scrobbles: Vec<Model>) {
    let mut ids: Vec<u32> = vec![];
    for scrobble in scrobbles {
        ids.push(scrobble.track_id);
    }
    ids.dedup();
    assert_eq!(ids.len(), 1, "More than 1 track has been created.");
}

#[tokio::test]
async fn match_entities() {
    common().await;

    let tracks1: Vec<TrackInfo> = vec![
        (vec!["Jennie", "Lalisa"], "The Ballad of Ace LeBaron"),
        (vec!["Jennie", "Lalisa"], "The Ballad of Ace Lebaron"),
        (vec!["Lalisa", "Jennie"], "the ballad of ace lebaron"),
        (vec!["lalisa", "jennie"], "THE BALLAD OF ACE LEBARON"),
    ];
    let amount1 = tracks1.len();

    let tracks2: Vec<TrackInfo> = vec![
        (vec!["Howard Shore", "Jeremy Soule"], "Party in the PRC"),
        (vec!["Howard Shore", "Jeremy Soule"], "party in the prc"),
        (vec!["Jeremy Soule", "Howard Shore"], "Party in The PRC"),
        (vec!["jeremy soule", "howard shore"], "Party In The PRC"),
        (
            vec![
                "jeremy soule",
                "howard shore",
                "Jeremy Soule",
                "Howard Shore",
                "JEREMY SOULE",
                "HOWARD SHORE",
            ],
            "Party In The PRC",
        ),
    ];
    let amount2 = tracks2.len();

    // first one by one
    let mut scrobbles = vec![];
    for write in create_scrobble_writes(tracks1).clone() {
        let db_result =
            database::repository::create_scrobbles(vec![write.clone()], false).await.unwrap();
        scrobbles.push(db_result[&write].clone())
    }
    assert_eq!(scrobbles.len(), amount1, "The incorrect amount of scrobbles has been generated");
    assert_all_same_track(scrobbles);

    // now all at once
    // TODO why does this overflow the stack???
    let db_result = database::repository::create_scrobbles(create_scrobble_writes(tracks2), false)
        .await
        .unwrap();
    assert_eq!(db_result.len(), amount2, "The incorrect amount of scrobbles has been generated");
    let scrobbles = db_result.values().map(|x| x.to_owned()).collect();
    assert_all_same_track(scrobbles);
}

#[tokio::test]
async fn parse_multi_artists() {
    common().await;

    let tracks1: Vec<TrackInfo> = vec![
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
    let amount = tracks1.len();

    let db_result = database::repository::create_scrobbles(create_scrobble_writes(tracks1), false)
        .await
        .unwrap();
    assert_eq!(db_result.len(), amount, "The incorrect amount of scrobbles has been generated");
    let scrobbles = db_result.values().map(|x| x.to_owned()).collect();
    assert_all_same_track(scrobbles);
}
