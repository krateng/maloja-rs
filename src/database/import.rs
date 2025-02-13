use std::error::Error;
use std::ffi::OsString;
use std::{fs, io};
use std::path::PathBuf;
use log::info;
use sea_orm::{ConnectionTrait, DatabaseConnection, DbErr, EntityTrait};
use sea_orm::ActiveValue::Set;
use sea_orm::ColumnType::Json;
use serde_json::Value;
use serde::Deserialize;
use crate::configuration::FOLDERS;
use crate::configuration::logging::display_path;
use crate::database::connect;
use crate::database::repository::create_scrobbles;
use crate::entity::{
    scrobble::Entity as ScrobbleEntity, scrobble::ActiveModel as ScrobbleModel,
    track::Entity as TrackEntity, track::ActiveModel as TrackModel,
    album::Entity as AlbumEntity, album::ActiveModel as AlbumModel,
    artist::Entity as ArtistEntity, artist::ActiveModel as ArtistModel,
};
use crate::entity::artist::ArtistWrite;
use crate::entity::scrobble::ScrobbleWrite;
use crate::entity::track::TrackWrite;

pub async fn import() -> Result<(i32, i32), io::Error> {
    let import_folder = FOLDERS.data.join("import");

    let (mut imported, mut failed): (i32, i32) = (0, 0);
    if import_folder.exists() {
        for entry in fs::read_dir(import_folder)? {
            let entry = entry?;
            let result = match entry.file_name().to_str() {
                Some("maloja_export.json") => { import_maloja(entry.path()).await }
                _ => {
                    failed += 1;
                    continue
                }
            };
            imported += 1;
        }
    }
    Ok((imported, failed))
}

#[derive(Deserialize, Debug, Clone)]
struct MalojaExport {
    maloja: Value,
    scrobbles: Vec<MalojaExportScrobble>,
}
#[derive(Deserialize, Debug, Clone)]
struct MalojaExportScrobble {
    time: i64,
    track: MalojaExportTrack,
    duration: Option<u32>,
    origin: Option<String>
}
#[derive(Deserialize, Debug, Clone)]
struct MalojaExportTrack {
    artists: Vec<String>,
    title: String,
    album: Option<MalojaExportAlbum>,
    length: Option<u32>,
}
#[derive(Deserialize, Debug, Clone)]
struct MalojaExportAlbum {
    artists: Option<Vec<String>>,
    albumtitle: String,
}


pub async fn import_maloja(file: PathBuf) -> Result<(), io::Error> {


    info!("Importing from Maloja export {}. This could take a while...", display_path(&file));

    let parsed: MalojaExport = serde_json::from_reader(
        fs::File::open(file)?
    )?;

    let db = connect().await;
    
    let scrobbles: Vec<ScrobbleWrite> = parsed.scrobbles.into_iter().map(|scrobble| {
        ScrobbleWrite {
            timestamp: scrobble.time,
            track: TrackWrite {
                id: None,
                title: Some(scrobble.track.title),
                primary_artists: Some(scrobble.track.artists.into_iter().map(|x| {
                    ArtistWrite {
                        id: None,
                        name: Some(x),
                        mbid: None,
                        spotify_id: None,
                    }
                }).collect()),
                secondary_artists: None,
                length: scrobble.track.length,
                mbid: None,
                spotify_id: None,
            },
            origin: scrobble.origin,
            duration: scrobble.duration,
        }
    }).collect();

   
    
    create_scrobbles(scrobbles, false).await;

    Ok(())
    
}