use crate::configuration::FOLDERS;
use crate::database;
use crate::entity::{
    album::Entity as Album,
    artist::Entity as Artist,
    scrobble::Entity as Scrobble,
    track::Entity as Track,
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
    //println!("Creating tables...");
    let builder = db.get_database_backend();
    let schema = Schema::new(builder);
    //println!("Schema: {:?}", schema);
    //let statement_builder = schema.create_table_from_entity(Album).if_not_exists();
    //let statement = builder.build(statement_builder);
    //println!("Statement: {:?}", statement);
    //db.execute(statement).await.unwrap();
    //println!("Done");
}
