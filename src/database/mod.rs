pub(crate) mod repository;
pub(crate) mod views;
pub(crate) mod import;

use crate::configuration::FOLDERS;
use crate::entity::{
    album::Entity as Album,
    artist::Entity as Artist,
    scrobble::Entity as Scrobble,
    track::Entity as Track,
    track_artist::Entity as TrackArtist,
    album_artist::Entity as AlbumArtist,
};
use sea_orm::{ConnectOptions, ConnectionTrait, Database, DatabaseConnection, DbBackend, DbConn, Schema};
use std::path::PathBuf;

fn get_database_path() -> PathBuf {
    FOLDERS.data.join("maloja.sqlite")
}

pub async fn init_db() {

    let db = connect().await;
    assert_eq!(db.get_database_backend(), DbBackend::Sqlite);
    
    log::info!("Checking Database schema...");
    create_tables(&db).await;
    log::info!("Checking imports...");
    import::import().await;
}

pub async fn connect() -> DatabaseConnection {
    let dbfile = get_database_path().display().to_string();
    let dbstring = format!("sqlite://{}?mode=rwc", dbfile);
    let mut dboptions = ConnectOptions::new(dbstring);
    dboptions.sqlx_logging(false);
    let db: DatabaseConnection = Database::connect(dboptions).await.unwrap();
    db

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
