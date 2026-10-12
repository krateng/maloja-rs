use crate::database::errors::MalojaError;
use crate::database::{connect, mark_db_write, normalize};
use crate::entity::{AlbumActiveModel, Album, AlbumModel, AlbumWrite, AlbumColumn, ArtistActiveModel, Artist, ArtistModel, ArtistWrite, ArtistColumn, TrackActiveModel, Track, TrackModel, TrackWrite, TrackColumn, ScrobbleActiveModel, Scrobble, ScrobbleModel, ScrobbleWrite, ScrobbleColumn, AlbumArtistActiveModel, AlbumArtist, TrackArtistActiveModel, TrackArtist, ArtistWriteContext, ScrobbleRead, TrackArtistModel, AlbumArtistModel};
use log::debug;
use sea_orm::ActiveValue::Set;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, NotSet, QueryFilter};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use crate::database::repository::resolve_scrobble_ids;
use crate::database::repository::writes_to_models::{album_write_to_model, artist_write_to_model, scrobble_write_to_model, track_write_to_model};

/// How many entities should be inserted into the Database in one go
const BATCH_SIZE: usize = 250;


// alrighty this time we're doing it organized from the start unlike the python monstrosity
// this is totally gonna work this time lmao
// link the relevant xkcd here


// These functions take the write with the already resolved IDs of the other entity needed to uniquely identify smth
// and return a full HashMap with IDs for the entities that already exist and None for the ones that don't

#[allow(clippy::collapsible_else_if)]
#[rustfmt::skip]
async fn get_existing_artist_ids(input: Vec<ArtistWrite>, db: &DatabaseConnection) -> Result<HashMap<ArtistWrite, Option<u32>>, MalojaError> {

    type UniqueArtistIdentifier = Option<String>;
    // write, identifier, resulting ID
    let mut all_info: Vec<(ArtistWrite, UniqueArtistIdentifier, Option<u32>)> = input.into_iter()
        .map(|write| (write.clone(), write.name.map(|x| normalize(x.as_str())), None) ).collect();

    // if the user specified an entity by its ID, we just check if it exists
    // no other fields are checked
    let ids: HashSet<u32> = all_info.iter().filter_map(|(write, _, _)| write.id).collect();
    // TODO

    // get all artists that fit in name
    let names_normalized: HashSet<String> = all_info.iter()
        .filter(|(_, _, result)| result.is_none()) // ignore the ones already matched
        .map(|(_, identifier, _)| -> Result<_,_> { identifier.clone().ok_or_else(|| MalojaError::MissingField { field: "name".to_string() }) }) // get all titles, error if missing title
        .collect::<Result<HashSet<_>, _>>()?;
    let db_result: Vec<ArtistModel> = Artist::find()
        .filter(ArtistColumn::NameNormalized.is_in(names_normalized))
        .all(db).await?;

    for artist in db_result {
        let unique: UniqueArtistIdentifier = Some(artist.name_normalized);
        for (_, identifier, result) in &mut all_info {
            if *identifier == unique {
                *result = Some(artist.id);
            }
        }
    }

    let result: HashMap<ArtistWrite, Option<u32>> = all_info.into_iter().map(|(write, _, result)| (write, result)).collect();
    Ok(result)
}

#[allow(clippy::collapsible_else_if)]
#[rustfmt::skip]
async fn get_existing_album_ids(input: Vec<(AlbumWrite, HashSet<u32>)>, db: &DatabaseConnection) -> Result<HashMap<AlbumWrite, Option<u32>>, MalojaError> {

    type UniqueAlbumIdentifier = (Option<String>, HashSet<u32>);
    // write, identifier, resulting ID
    let mut all_info: Vec<(AlbumWrite, UniqueAlbumIdentifier, Option<u32>)> = input.into_iter()
        .map(|(write, artist_ids)| (write.clone(), (write.album_title.map(|x| normalize(x.as_str())), artist_ids), None) ).collect();

    // if the user specified an entity by its ID, we just check if it exists
    // no other fields are checked
    let ids: HashSet<u32> = all_info.iter().filter_map(|(write, _, _)| write.id).collect();
    // TODO

    // get all albums that fit in name
    // not elegant, but doing some quick rust filtering seems more reasonable than crazy db queries
    // this is a one user application after all
    let titles_normalized: HashSet<String> = all_info.iter()
        .filter(|(_, _, result)| result.is_none()) // ignore the ones already matched
        .map(|(_, identifier, _)| -> Result<_,_> { identifier.clone().0.ok_or_else(|| MalojaError::MissingField { field: "album_title".to_string() }) }) // get all titles, error if missing title
        .collect::<Result<HashSet<_>, _>>()?;
    let db_result: Vec<(AlbumModel, Vec<AlbumArtistModel>)> = Album::find()
        .filter(AlbumColumn::AlbumTitleNormalized.is_in(titles_normalized))
        .find_with_related(AlbumArtist)
        .all(db).await?;

    for (album, album_artists) in db_result {
        let unique: UniqueAlbumIdentifier = (Some(album.album_title_normalized), album_artists.iter().map(|x| x.artist_id).collect::<HashSet<_>>());
        for (_, identifier, result) in &mut all_info {
            if *identifier == unique {
                *result = Some(album.id);
            }
        }
    }

    let result: HashMap<AlbumWrite, Option<u32>> = all_info.into_iter().map(|(write, _, result)| (write, result)).collect();
    Ok(result)
}

#[allow(clippy::collapsible_else_if)]
#[rustfmt::skip]
async fn get_existing_track_ids(input: Vec<(TrackWrite, HashSet<u32>)>, db: &DatabaseConnection) -> Result<HashMap<TrackWrite, Option<u32>>, MalojaError> {

    type UniqueTrackIdentifier = (Option<String>, HashSet<u32>);
    // write, identifier, resulting ID
    let mut all_info: Vec<(TrackWrite, UniqueTrackIdentifier, Option<u32>)> = input.into_iter()
        .map(|(write, artist_ids)| (write.clone(), (write.title.map(|x| normalize(x.as_str())), artist_ids), None) ).collect();

    // if the user specified an entity by its ID, we just check if it exists
    // no other fields are checked
    let ids: HashSet<u32> = all_info.iter().filter_map(|(write, _, _)| write.id).collect();
    // TODO

    // get all tracks that fit in name
    // not elegant, but doing some quick rust filtering seems more reasonable than crazy db queries
    // this is a one user application after all
    let titles_normalized: HashSet<String> = all_info.iter()
        .filter(|(_, _, result)| result.is_none()) // ignore the ones already matched
        .map(|(_, identifier, _)| -> Result<_,_> { identifier.clone().0.ok_or_else(|| MalojaError::MissingField { field: "title".to_string() }) }) // get all titles, error if missing title
        .collect::<Result<HashSet<_>, _>>()?;

    let db_result: Vec<(TrackModel, Vec<TrackArtistModel>)> = Track::find()
        .filter(TrackColumn::TitleNormalized.is_in(titles_normalized))
        .find_with_related(TrackArtist)
        .all(db).await?;

    for (track, track_artists) in db_result {
        let unique: UniqueTrackIdentifier = (Some(track.title_normalized), track_artists.iter().map(|x| x.artist_id).collect::<HashSet<_>>());
        for (_, identifier, result) in &mut all_info {
            if *identifier == unique {
                *result = Some(track.id);
            }
        }
    }

    let result: HashMap<TrackWrite, Option<u32>> = all_info.into_iter().map(|(write, _, result)| (write, result)).collect();
    Ok(result)
}

// These functions take the writes and resolve them all to IDs, either by matching or creating new

#[allow(clippy::collapsible_else_if)]
#[rustfmt::skip]
pub async fn get_or_create_artists(input: Vec<ArtistWrite>) -> Result<HashMap<ArtistWrite, u32>, MalojaError> {
    let db = connect().await?;

    // fill in existing ones
    let artist_info: Vec<ArtistWrite> = input.clone();
    let existing = get_existing_artist_ids(artist_info, &db).await?;

    // put the ones we already have in the result
    let mut result: HashMap<ArtistWrite, u32> = existing.into_iter()
        .filter_map(|(k, v)| {
            v.map(|id| (k, id))
        }).collect();

    // create inserts for all remaining ones
    let inserts: HashMap<ArtistWrite, ArtistActiveModel> = input
        .into_iter()
        .filter(|write| !result.contains_key(write))
        .map(|a| {
            let mut model = artist_write_to_model(a.clone());
            (a, model)
        }).collect();

    // not as pretty as batch inserts, but to keep track of which model got which ID (sqlite doesnt guarantee order),
    // we do it all individually
    // using a txn should make the performance cost acceptable with sqlite, and we rarely insert lots of stuff
    // expect for the initial import
    let amount_inserts = &inserts.len();
    for (write, insert) in inserts {
        let db_result = Artist::insert(insert).exec(&db).await?;
        result.insert(write, db_result.last_insert_id);
    }

    mark_db_write();
    debug!("Inserted {:?} artists", amount_inserts);
    Ok(result)
}


#[allow(clippy::collapsible_else_if)]
#[rustfmt::skip]
pub async fn get_or_create_albums(input: Vec<AlbumWrite>) -> Result<HashMap<AlbumWrite, u32>, MalojaError> {
    let db = connect().await?;

    let album_artist_write_map: HashMap<AlbumWrite, HashSet<ArtistWriteContext>> = input.clone().into_iter()
        .map(|x| (
            x.clone(),
            x.album_artists.unwrap_or(vec![]).into_iter().collect()
        )).collect();
    let artist_writes: Vec<ArtistWrite> = album_artist_write_map.iter().flat_map(|(_, v)| v.clone()).map(|x| x.into_artist_write()).collect();
    let artist_id_map: HashMap<ArtistWrite, u32> = get_or_create_artists(artist_writes).await?;
    // carry context info with us for later insertion
    let album_artist_id_map: HashMap<AlbumWrite, HashSet<(u32, ArtistWriteContext)>> = album_artist_write_map
        .into_iter().map(|(al,ar)| (al, ar.into_iter().map(|aw| (artist_id_map[&aw.clone().into_artist_write()], aw)).collect())).collect();

    // fill in existing ones
    let album_info: Vec<(AlbumWrite, HashSet<u32>)> = album_artist_id_map.iter().map(|(write, artists)|
        (write.clone(), artists.iter().map(|(id, _)| *id).collect())
    ).collect();
    let existing = get_existing_album_ids(album_info, &db).await?;

    // put the ones we already have in the result
    let mut result: HashMap<AlbumWrite, u32> = existing.into_iter()
        .filter_map(|(k, v)| {
            v.map(|id| (k, id))
        }).collect();

    // create inserts for all remaining ones
    let inserts: HashMap<AlbumWrite, AlbumActiveModel> = album_artist_id_map
        .into_iter()
        .filter(|(write, _)| !result.contains_key(write))
        .map(|(a,aid)| {
            let mut model = album_write_to_model(a.clone());
            (a, model)
        }).collect();


    // not as pretty as batch inserts, but to keep track of which model got which ID (sqlite doesnt guarantee order),
    // we do it all individually
    // using a txn should make the performance cost acceptable with sqlite, and we rarely insert lots of stuff
    // expect for the initial import
    let amount_inserts = &inserts.len();
    for (write, insert) in inserts {
        let db_result = Album::insert(insert).exec(&db).await?;
        result.insert(write, db_result.last_insert_id);
    }

    mark_db_write();
    debug!("Inserted {:?} albums", amount_inserts);
    Ok(result)
}

#[allow(clippy::collapsible_else_if)]
#[rustfmt::skip]
pub async fn get_or_create_tracks(input: Vec<TrackWrite>) -> Result<HashMap<TrackWrite, u32>, MalojaError> {
    let db = connect().await?;

    let track_artist_write_map: HashMap<TrackWrite, HashSet<ArtistWriteContext>> = input.clone().into_iter()
        .map(|x| (
        x.clone(),
        x.artists.unwrap_or(vec![]).into_iter().collect()
    )).collect();
    let artist_writes: Vec<ArtistWrite> = track_artist_write_map.iter().flat_map(|(_, v)| v.clone()).map(|x| x.into_artist_write()).collect();
    let artist_id_map: HashMap<ArtistWrite, u32> = get_or_create_artists(artist_writes).await?;
    // carry context info with us for later insertion
    let track_artist_id_map: HashMap<TrackWrite, HashSet<(u32, ArtistWriteContext)>> = track_artist_write_map
        .into_iter().map(|(tr,ar)| (tr, ar.into_iter().map(|aw| (artist_id_map[&aw.clone().into_artist_write()], aw)).collect())).collect();

    let track_album_write_map: HashMap<TrackWrite, Option<AlbumWrite>> = input.clone().into_iter().map(|x| (x.clone(), x.album)).collect();
    let album_writes: Vec<AlbumWrite> = track_album_write_map.iter().filter_map(|(_, v)| v.clone()).collect();
    let album_id_map: HashMap<AlbumWrite, u32> = get_or_create_albums(album_writes).await?;
    let track_album_id_map: HashMap<TrackWrite, Option<u32>> = track_album_write_map
        .into_iter().map(|(tr,al)| (tr, al.map(|a| album_id_map[&a]))).collect();

    // fill in existing ones
    let track_info: Vec<(TrackWrite, HashSet<u32>)> = track_artist_id_map.iter().map(|(write, artists)|
        (write.clone(), artists.iter().map(|(id, _)| *id).collect())
    ).collect();
    let existing = get_existing_track_ids(track_info, &db).await?;

    // put the ones we already have in the result
    let mut result: HashMap<TrackWrite, u32> = existing.into_iter()
        .filter_map(|(k, v)| {
            v.map(|id| (k, id))
        }).collect();

    // create inserts for all remaining ones
    let inserts: HashMap<TrackWrite, TrackActiveModel> = track_album_id_map
        .into_iter()
        .filter(|(write, _)| !result.contains_key(write))
        .map(|(t,aid)| {
            let mut model = track_write_to_model(t.clone());
            model.album_id = aid.map(|x| Set(Some(x))).unwrap_or(NotSet);
            (t, model)
        }).collect();

    // not as pretty as batch inserts, but to keep track of which model got which ID (sqlite doesnt guarantee order),
    // we do it all individually
    // using a txn should make the performance cost acceptable with sqlite, and we rarely insert lots of stuff
    // expect for the initial import
    let amount_inserts = &inserts.len();
    for (write, insert) in inserts {
        let db_result = Track::insert(insert).exec(&db).await?;
        result.insert(write, db_result.last_insert_id);
    }

    mark_db_write();
    debug!("Inserted {:?} tracks", amount_inserts);
    Ok(result)
}

#[allow(clippy::collapsible_else_if)]
#[rustfmt::skip]
pub async fn create_scrobbles(input: Vec<ScrobbleWrite>, _fail_on_existing: bool) -> Result<HashMap<ScrobbleWrite, i64>, MalojaError> {
    // this one is a bit different that the other entity ones because we never supply a scrobblewrite
    // as part of another entity to either create or fetch - scrobbles are only ever created (or patched?)
    let db = connect().await?;

    let scrobble_track_write_map: HashMap<ScrobbleWrite, TrackWrite> = input.clone().into_iter().map(|x| (x.clone(), x.track)).collect();
    let track_writes: Vec<TrackWrite> = scrobble_track_write_map.iter().map(|(_, v)| v.clone()).collect();
    let track_id_map: HashMap<TrackWrite, u32> = get_or_create_tracks(track_writes).await?;
    let scrobble_track_id_map: HashMap<ScrobbleWrite, u32> = scrobble_track_write_map
        .into_iter().map(|(s,t)| (s, track_id_map[&t]) ).collect();

    let inserts: HashMap<ScrobbleWrite, ScrobbleActiveModel> = scrobble_track_id_map
        .into_iter()
        .map(|(w,tid)| {
            let mut model = scrobble_write_to_model(w.clone());
            model.track_id = Set(tid);
            (w, model)
        }).collect();

    let mut result: HashMap<ScrobbleWrite, i64> = HashMap::new();

    // ok so for scrobbles, where the ID is set before, we can do batch inserts and match by id
    let amount_inserts = &inserts.len();
    for chunk in inserts.iter().collect::<Vec<_>>().chunks(BATCH_SIZE) {
        let chunk_inserts = chunk.to_vec();
        let actual_inserts: Vec<ScrobbleActiveModel> = chunk_inserts.into_iter().map(|(w,am)| am.to_owned()).collect();
        let db_result: Vec<i64> = Scrobble::insert_many(actual_inserts).exec_with_returning_keys(&db).await?;
        for id in db_result {
            // we basically just get the key to match by input ts even tho we already have that
            // this in the model and could just build a result map based on that
            // this is just for pattern accuracy so we actually confirm each id has a db entry
            let key: ScrobbleWrite = inserts.iter()
                .find(|(w,am)| am.timestamp.clone() == Set(id) ).unwrap().0.clone();
            result.insert(key, id);
        }
    }
    mark_db_write();
    debug!("Inserted {:?} Scrobbles", amount_inserts);
    Ok(result)
}



pub async fn create_scrobbles_return_reads(input: Vec<ScrobbleWrite>, _fail_on_existing: bool) -> Result<Vec<ScrobbleRead>, MalojaError> {
    let map = create_scrobbles(input.clone(), _fail_on_existing).await?;
    let db = connect().await?;
    let reads_map: HashMap<i64, ScrobbleRead> = resolve_scrobble_ids(map.values().cloned().collect(), &db).await?;
    let mut reads = vec![];
    for input_write in input {
        let ts = map.get(&input_write).expect("Map should have all inputs");
        let read = reads_map.get(&ts).expect("Reads map should have all IDs");
        reads.push(read.to_owned());
    }
    Ok(reads)
}

