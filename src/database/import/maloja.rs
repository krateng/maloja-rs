use std::collections::{HashMap, HashSet};
use crate::configuration::logging::display_path;
use crate::database::errors::MalojaError;
use crate::entity::album::AlbumWrite;
use crate::entity::scrobble::ScrobbleWrite;
use crate::entity::track::TrackWrite;
use log::info;
use std::fs;
use std::path::PathBuf;
use crate::configuration::app_state::AppState;
use crate::database::repository::get_or_create::AttributedScrobbleWrite;
use crate::entity::ArtistWriteContext;
use maloja_dto::export::{MalojaExport, MalojaScrobbleOrigin};


// TODO
// we might want to rework this to not have the import itself call anything within the DB but just return
// write / origin data and the db inserts it, for de-spaghettification

pub async fn import_maloja(file: PathBuf, state: &AppState) -> Result<(), MalojaError> {
    info!("Importing from Maloja export {}. This could take a while...", display_path(&file));

    let parsed: MalojaExport = serde_json::from_reader(fs::File::open(file)?)?;

    let clients: HashSet<MalojaScrobbleOrigin> = parsed.scrobbles.iter().filter_map(|s| s.origin.clone()).collect();
    let mut client_map: HashMap<MalojaScrobbleOrigin, Option<u32>> = HashMap::new();
    for client in clients {
        let client_id = match &client {
            MalojaScrobbleOrigin::AttributedClient{ identifier } => {
                Some(state.db.import_client(identifier.clone()).await?)
            }
            _ => None,
        };
        client_map.insert(client, client_id);
    }

    let scrobbles: Vec<AttributedScrobbleWrite> = parsed
        .scrobbles
        .into_iter()
        .map(|scrobble| {
            AttributedScrobbleWrite {
                write: ScrobbleWrite {
                    timestamp: Some(scrobble.time),
                    track: TrackWrite {
                        id: None,
                        title: Some(scrobble.track.title),
                        artists: Some(
                            scrobble
                                .track
                                .artists
                                .into_iter()
                                .map(|a| ArtistWriteContext {
                                    id: None,
                                    name: Some(a),
                                    alias: None,
                                    primary: true,
                                    mbid: None,
                                    spotify_id: None,
                                })
                                .collect(),
                        ),
                        album: scrobble.track.album.map(|al| AlbumWrite {
                            id: None,
                            album_title: Some(al.album_title),
                            album_artists: al.artists.map(|aas| {
                                aas.iter()
                                    .map(|aa| {
                                        // outer map is to unwrap the option, inner map an actual vector map
                                        ArtistWriteContext {
                                            id: None,
                                            name: Some(aa.to_owned()),
                                            alias: None,
                                            primary: true,
                                            mbid: None,
                                            spotify_id: None,
                                        }
                                    })
                                    .collect()
                            }),
                            mbid: None,
                            spotify_id: None,
                        }),
                        track_length: scrobble.track.length,
                        mbid: None,
                        spotify_id: None,
                    },
                    listen_duration: scrobble.duration,
                },
                client_id: scrobble.origin.and_then(|x| *client_map.get(&x).unwrap()),
            }

        })
        .collect();

    state.db.create_scrobbles(scrobbles, false, false).await?;

    Ok(())
}
