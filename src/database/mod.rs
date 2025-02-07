use crate::configuration::FOLDERS;
use crate::entity::{
    album::Entity as Album,
    artist::Entity as Artist,
    scrobble::Entity as Scrobble,
    track::Entity as Track,
    track_artist::Entity as TrackArtist,
    album_artist::Entity as AlbumArtist,
};
use sea_orm::{ConnectionTrait, Database, DatabaseConnection, DbBackend, DbConn, Schema};
use std::path::PathBuf;

fn get_database_path() -> PathBuf {
    FOLDERS.data.join("malojadb.sqlite")
}

pub async fn init_db() {
    let dbfile = get_database_path().display().to_string();
    let dbstring = format!("sqlite://{}?mode=rwc", dbfile);

    let db: DatabaseConnection = Database::connect(dbstring).await.unwrap();
    assert_eq!(db.get_database_backend(), DbBackend::Sqlite);
    create_tables(&db).await;
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
