pub(crate) mod repository;
pub(crate) mod views;
pub(crate) mod import;

use std::io::Error;
use crate::configuration::FOLDERS;
use crate::entity::{
    album::Entity as Album,
    artist::Entity as Artist,
    scrobble::Entity as Scrobble,
    track::Entity as Track,
    track_artist::Entity as TrackArtist,
    album_artist::Entity as AlbumArtist,
};
use sea_orm::{ConnectOptions, ConnectionTrait, Database, DatabaseConnection, DbBackend, DbConn, DbErr, Schema, Statement};
use std::path::PathBuf;

fn get_database_path() -> PathBuf {
    FOLDERS.data.join("maloja.sqlite")
}

pub async fn init_db() -> Result<(), DbErr> {

    let db = connect().await?;
    assert_eq!(db.get_database_backend(), DbBackend::Sqlite);

    let version: String = db.query_one(Statement::from_string(
        sea_orm::DatabaseBackend::Sqlite,
        "SELECT sqlite_version();".to_owned(),
    )).await?.unwrap().try_get("", "sqlite_version()")?;

    log::info!("Using SQLite {}", version);

    log::info!("Checking Database schema...");
    create_tables(&db).await;
    log::info!("Checking imports...");
    match import::import().await {
        Ok((imported, failed)) => {
            log::info!("Imported {} files, failed {}.", imported, failed);
        }
        Err(_) => {
            log::error!("Failed to check for imports...");
        }
    };
    
    Ok(())
}

pub async fn connect() -> Result<DatabaseConnection, DbErr> {
    let dbfile = get_database_path().display().to_string();
    let dbstring = format!("sqlite://{}?mode=rwc", dbfile);
    let mut dboptions = ConnectOptions::new(dbstring);
    dboptions.sqlx_logging(false);
    let db: DatabaseConnection = Database::connect(dboptions).await?;
    Ok(db)
}

pub async fn create_tables(db: &DbConn) {

    create_table(db, Scrobble).await;
    create_table(db, Track).await;
    create_table(db, Artist).await;
    create_table(db, Album).await;

    create_table(db, TrackArtist).await;
    create_table(db, AlbumArtist).await;
}

async fn create_table<E: sea_orm::EntityTrait>(db: &DbConn, entity: E) {
    let backend = db.get_database_backend();
    let schema = Schema::new(backend);
    let stmt = schema.create_table_from_entity(entity).if_not_exists().to_owned();
    // TODO: wtf is even going on here
    let statement = backend.build(&stmt);
    let result = db.execute(statement).await.expect("wut");

}
