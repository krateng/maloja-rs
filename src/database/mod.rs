pub mod errors;
pub mod import;
pub mod repository;
pub mod views;
mod transform;

use crate::database::errors::MalojaError;
use crate::entity::{
    album::Entity as Album, album_artist::Entity as AlbumArtist, artist::Entity as Artist,
    scrobble::Entity as Scrobble, track::Entity as Track, track_artist::Entity as TrackArtist,
};
use import::import;
use sea_orm::{
    ConnectOptions, ConnectionTrait, Database, DatabaseConnection, DbBackend, DbConn,
    Schema, Statement,
};
use std::path::PathBuf;
use crate::configuration::app_state::{AppState, ApplicationFolders};

fn get_database_path(folders: &ApplicationFolders) -> PathBuf {
    folders.data.join("maloja.sqlite")
}

pub async fn init_db(state: &AppState) -> Result<(), MalojaError> {
    let db = connect(&state.folders).await?;
    assert_eq!(db.get_database_backend(), DbBackend::Sqlite);

    let version: String = db
        .query_one_raw(Statement::from_string(
            sea_orm::DatabaseBackend::Sqlite,
            "SELECT sqlite_version();".to_owned(),
        ))
        .await?
        .unwrap()
        .try_get("", "sqlite_version()")?;

    log::info!("Using SQLite {}", version);

    log::info!("Checking Database schema...");
    create_tables(&db).await?;
    log::info!("Checking imports...");
    match import(state).await {
        Ok((imported, failed)) => {
            log::info!("Imported {} files, failed {}.", imported, failed);
        }
        Err(_) => {
            log::error!("Failed to check for imports...");
        }
    };

    Ok(())
}

/// This function should be called every time the database has been written to and is in a new consistent state
/// (so not after every single atomic write, but logical write operations)
pub fn mark_db_write() {
    // for the future
}

pub async fn connect(folders: &ApplicationFolders) -> Result<DatabaseConnection, MalojaError> {
    let dbfile = get_database_path(folders).display().to_string();
    let dbstring = format!("sqlite://{}?mode=rwc", dbfile);
    let mut dboptions = ConnectOptions::new(dbstring);
    dboptions.sqlx_logging(false);
    match Database::connect(dboptions).await {
        Ok(conn) => {
            conn.execute_unprepared("PRAGMA foreign_keys = ON").await?;
            Ok(conn)
        }
        Err(e) => Err(MalojaError::DatabaseConnectionError { message: e.to_string() }),
    }
}

pub async fn create_tables(db: &DbConn) -> Result<(), MalojaError> {
    // TODO: proper migrations

    create_table(db, Scrobble).await?;
    create_table(db, Track).await?;
    create_table(db, Artist).await?;
    create_table(db, Album).await?;

    create_table(db, TrackArtist).await?;
    create_table(db, AlbumArtist).await?;

    Ok(())
}

async fn create_table<E: sea_orm::EntityTrait>(db: &DbConn, entity: E) -> Result<(), MalojaError> {
    let backend = db.get_database_backend();
    let schema = Schema::new(backend);
    let mut stmt = schema.create_table_from_entity(entity);
    stmt.if_not_exists();
    let _result = db.execute(&stmt).await?;
    // TODO: STRICT types
    Ok(())
}

/// Shared function to normalize titles and names for the purpose of duplication detection
pub fn normalize(input: &str) -> String {
    input.to_lowercase().replace("_", "-").replace(" ", "-")
}
