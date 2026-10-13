use crate::database::errors::MalojaError;
use crate::database::{connect, mark_db_write, normalize};
use crate::entity::{AlbumActiveModel, Album, AlbumModel, AlbumWrite, AlbumColumn, ArtistActiveModel, Artist, ArtistModel, ArtistWrite, ArtistColumn, TrackActiveModel, Track, TrackModel, TrackWrite, TrackColumn, ScrobbleActiveModel, Scrobble, ScrobbleModel, ScrobbleWrite, ScrobbleColumn, AlbumArtistActiveModel, AlbumArtist, TrackArtistActiveModel, TrackArtist, ArtistWriteContext, ScrobbleRead, TrackArtistModel, AlbumArtistModel};
use log::debug;
use sea_orm::ActiveValue::Set;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, NotSet, QueryFilter, TransactionSession, TransactionTrait};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::task::id;
use crate::database::repository::resolve_scrobble_ids;
use crate::database::repository::writes_to_models::{album_write_to_model, artist_write_to_model, scrobble_write_to_model, track_write_to_model};
use crate::entity::album::UniqueAlbum;
use crate::entity::artist::UniqueArtist;
use crate::entity::track::UniqueTrack;

/// How many entities should be inserted into the Database in one go
const BATCH_SIZE: usize = 2048;


// alrighty this time we're doing it organized from the start unlike the python monstrosity
// this is totally gonna work this time lmao
// link the relevant xkcd here


// These functions take the write with the already resolved IDs of the other entity needed to uniquely identify smth
// and return a full HashMap with IDs for the entities that already exist and None for the ones that don't

#[allow(clippy::collapsible_else_if)]
#[rustfmt::skip]
async fn get_existing_artist_ids(input: HashSet<UniqueArtist>, db: &DatabaseConnection) -> Result<HashMap<UniqueArtist, Option<u32>>, MalojaError> {

    let mut result: HashMap<UniqueArtist, Option<u32>> = input
        .into_iter().map(|u| (u, None)).collect();
    // if the user specified an entity by its ID, we just check if it exists
    // no other fields are checked
    // TODO

    // get all artists that fit in name
    let names_normalized: HashSet<String> = result.iter() // ignore the ones already matched
        .filter_map(|(u, _)| {
            match u {
                UniqueArtist::ById(_) => None,
                UniqueArtist::ByInfo { normalized_name } => Some(normalized_name.clone()),
            }
        }).collect();

    let db_result: Vec<ArtistModel> = Artist::find()
        .filter(ArtistColumn::NameNormalized.is_in(names_normalized))
        .all(db).await?;

    for artist in db_result {
        let identifier = UniqueArtist::ByInfo {
            normalized_name: artist.name_normalized
        };
        result.insert(identifier, Some(artist.id));
    }

    Ok(result)
}

#[allow(clippy::collapsible_else_if)]
#[rustfmt::skip]
async fn get_existing_album_ids(input: HashSet<UniqueAlbum>, db: &DatabaseConnection) -> Result<HashMap<UniqueAlbum, Option<u32>>, MalojaError> {

    let mut result: HashMap<UniqueAlbum, Option<u32>> = input
        .into_iter().map(|u| (u, None)).collect();
    // if the user specified an entity by its ID, we just check if it exists
    // no other fields are checked
    // TODO

    // get all albums that fit in name
    // not elegant, but doing some quick rust filtering seems more reasonable than crazy db queries
    // this is a one user application after all
    let titles_normalized: HashSet<String> = result.iter() // ignore the ones already matched
        .filter_map(|(u, _)| {
            match u {
                UniqueAlbum::ById(_) => None,
                UniqueAlbum::ByInfo { normalized_title, .. } => Some(normalized_title.clone()),
            }
        }).collect();

    let db_result: Vec<(AlbumModel, Vec<AlbumArtistModel>)> = Album::find()
        .filter(AlbumColumn::AlbumTitleNormalized.is_in(titles_normalized))
        .find_with_related(AlbumArtist)
        .all(db).await?;

    for (album, album_artists) in db_result {
        let identifier = UniqueAlbum::ByInfo {
            normalized_title: album.album_title_normalized,
            artist_ids: album_artists.iter().map(|x| x.artist_id).collect()
        };
        result.insert(identifier, Some(album.id));
    }
    Ok(result)
}

#[allow(clippy::collapsible_else_if)]
#[rustfmt::skip]
async fn get_existing_track_ids(input: HashSet<UniqueTrack>, db: &DatabaseConnection) -> Result<HashMap<UniqueTrack, Option<u32>>, MalojaError> {

    let mut result: HashMap<UniqueTrack, Option<u32>> = input
        .into_iter().map(|u| (u, None)).collect();
    // if the user specified an entity by its ID, we just check if it exists
    // no other fields are checked
    // TODO

    // get all tracks that fit in name
    // not elegant, but doing some quick rust filtering seems more reasonable than crazy db queries
    // this is a one user application after all
    let titles_normalized: HashSet<String> = result.iter()
        .filter_map(|(u, _)| {
            match u {
                UniqueTrack::ById(_) => None,
                UniqueTrack::ByInfo{ normalized_title, .. } => Some(normalized_title.clone()),
            }
        }).collect();

    let db_result: Vec<(TrackModel, Vec<TrackArtistModel>)> = Track::find()
        .filter(TrackColumn::TitleNormalized.is_in(titles_normalized))
        .find_with_related(TrackArtist)
        .all(db).await?;

    for (track, track_artists) in db_result {
        let identifier = UniqueTrack::ByInfo {
            normalized_title: track.title_normalized,
            artist_ids: track_artists.iter().map(|x| x.artist_id).collect()
        };
        result.insert(identifier, Some(track.id));
    }
    Ok(result)
}

// These functions take the writes and resolve them all to IDs, either by matching or creating new

#[allow(clippy::collapsible_else_if)]
#[rustfmt::skip]
pub async fn get_or_create_artists(input: Vec<ArtistWrite>) -> Result<HashMap<ArtistWrite, u32>, MalojaError> {
    let db = connect().await?;

    let map_to_unique: HashMap<ArtistWrite, UniqueArtist> = input.into_iter()
        .map(|write| {
            let unique = write.unique()?;
            Ok((write, unique))
        }).collect::<Result<HashMap<_, _>, MalojaError>>()?;

    type ArtistInfo = (ArtistWrite,);
    let mut unique_map: HashMap<UniqueArtist, (ArtistInfo, Option<u32>)> = map_to_unique
        .iter().map(|(write, unique)| (
            unique.clone(),
            (
                (
                    write.clone(),
                ),
                None
            )
        )).collect();

    // fill in existing ones
    let unique_set: HashSet<UniqueArtist> = unique_map.keys().cloned().collect();
    let unique_result_map = get_existing_artist_ids(unique_set, &db).await?;
    unique_result_map.into_iter().for_each(|(u, r)| {
        if let Some(id) = r {
            unique_map.get_mut(&u).expect("Uniques from result should be in the originally provided map").1 = Some(id);
        }
    } );

    // create inserts for all remaining ones
    let inserts: HashMap<UniqueArtist, (ArtistActiveModel, )> = unique_map
        .iter()
        .filter(|(_, (_, result))| result.is_none())
        .map(|(unique, ((artist, ), result))| {
            let mut model = artist_write_to_model(artist.clone());
            (unique.clone(), (model,))
        }).collect();

    // not as pretty as batch inserts, but to keep track of which model got which ID (sqlite doesnt guarantee order),
    // we do it all individually
    // using a txn should make the performance cost acceptable with sqlite, and we rarely insert lots of stuff
    // expect for the initial import
    let txn = db.begin().await?;
    let amount_inserts = &inserts.len();
    for (unique, (insert, )) in inserts {
        let db_result = Artist::insert(insert).exec(&txn).await?;
        let artist_id = db_result.last_insert_id;
        unique_map.get_mut(&unique)
            .expect("Uniques from insert map should be in the originally provided map")
            .1 = Some(artist_id);
    }
    txn.commit().await?;

    mark_db_write();
    debug!("Inserted {:?} artists", amount_inserts);
    Ok(map_to_unique.into_iter().map(|(write, unique)| {
        (write, unique_map[&unique].1.expect("At this point no entities should be unresolved"))
    }).collect())
}


#[allow(clippy::collapsible_else_if)]
#[rustfmt::skip]
pub async fn get_or_create_albums(input: Vec<AlbumWrite>) -> Result<HashMap<AlbumWrite, u32>, MalojaError> {
    let db = connect().await?;

    let artist_writes: Vec<ArtistWrite> = input.iter().flat_map(|x| x.album_artists.clone().unwrap_or_else(Vec::new)).map(|x| x.into_artist_write()).collect();
    let artist_map = get_or_create_artists(artist_writes).await?;

    let map_to_unique: HashMap<AlbumWrite, UniqueAlbum> = input.into_iter()
        .map(|write| {
            let unique = write.unique(&artist_map)?;
            Ok((write, unique))
        }).collect::<Result<HashMap<_, _>, MalojaError>>()?;

    // album, artists
    type AlbumInfo = (AlbumWrite, HashMap<u32, ArtistWriteContext>);
    let mut unique_map: HashMap<UniqueAlbum, (AlbumInfo, Option<u32>)> = map_to_unique
        .iter().map(|(write, unique)| (
            unique.clone(),
            (
                (
                    write.clone(),
                    write.album_artists.clone().unwrap_or_else(Vec::new).iter().map(|x| {
                        (artist_map[&x.clone().into_artist_write()], x.clone())
                    }).collect::<HashMap<u32, ArtistWriteContext>>()
                ),
                None
            )
        )).collect();

    // fill in existing ones
    let unique_set: HashSet<UniqueAlbum> = unique_map.keys().cloned().collect();
    let unique_result_map = get_existing_album_ids(unique_set, &db).await?;
    unique_result_map.into_iter().for_each(|(u, r)| {
        if let Some(id) = r {
            unique_map.get_mut(&u).expect("Uniques from result should be in the originally provided map").1 = Some(id);
        }
    });

    // create inserts for all remaining ones
    let inserts: HashMap<UniqueAlbum, (AlbumActiveModel, Vec<AlbumArtistActiveModel>)> = unique_map
        .iter()
        .filter(|(_, (_, result))| result.is_none())
        .map(|(unique, ((album, artists), result))| {
            let mut model = album_write_to_model(album.clone());
            let album_artist_inserts = artists.into_iter().map(|(id, context)| {
                    AlbumArtistActiveModel {
                        album_id: NotSet,
                        artist_id: Set(*id),
                        primary: Set(context.primary),
                        artist_alias: Set(context.alias.clone()),
                    }
                }).collect();
            (unique.clone(), (model, album_artist_inserts))
        }).collect();

    // not as pretty as batch inserts, but to keep track of which model got which ID (sqlite doesnt guarantee order),
    // we do it all individually
    // using a txn should make the performance cost acceptable with sqlite, and we rarely insert lots of stuff
    // expect for the initial import
    let txn = db.begin().await?;
    let amount_inserts = &inserts.len();
    for (unique, (insert, mut album_artists)) in inserts {
        let db_result = Album::insert(insert).exec(&txn).await?;
        let album_id: u32 = db_result.last_insert_id;
        unique_map.get_mut(&unique)
            .expect("Uniques from insert map should be in the originally provided map")
            .1 = Some(album_id);

        for aux_insert in &mut album_artists {
            aux_insert.album_id = Set(album_id);
        }

        AlbumArtist::insert_many(album_artists).exec(&txn).await?;
    }
    txn.commit().await?;

    mark_db_write();
    debug!("Inserted {:?} albums", amount_inserts);
    Ok(map_to_unique.into_iter().map(|(write, unique)| {
        (write, unique_map[&unique].1.expect("At this point no entities should be unresolved"))
    }).collect())
}

#[allow(clippy::collapsible_else_if)]
#[rustfmt::skip]
pub async fn get_or_create_tracks(input: Vec<TrackWrite>) -> Result<HashMap<TrackWrite, u32>, MalojaError> {
    let db = connect().await?;

    let artist_writes: Vec<ArtistWrite> = input.iter().flat_map(|x| x.artists.clone().unwrap_or_else(Vec::new)).map(|x| x.into_artist_write()).collect();
    let album_writes: Vec<AlbumWrite> = input.iter().filter_map(|x| x.album.clone()).collect();
    let artist_map = get_or_create_artists(artist_writes).await?;
    let album_map = get_or_create_albums(album_writes).await?;

    let map_to_unique: HashMap<TrackWrite, UniqueTrack> = input.into_iter()
        .map(|write| {
            let unique = write.unique(&artist_map)?;
            Ok((write, unique))
        }).collect::<Result<HashMap<_, _>, MalojaError>>()?;

    // track, artists, album
    type TrackInfo = (TrackWrite, HashMap<u32, ArtistWriteContext>, Option<u32>);
    let mut unique_map: HashMap<UniqueTrack, (TrackInfo, Option<u32>)> = map_to_unique
        .iter().map(|(write, unique)| (
            // we will go through duplicates twice here and one write will be used as representative
            // for metadata that does not break identity (alias etc)
            unique.clone(),
            (
                (
                    write.clone(),
                    write.artists.clone().unwrap_or_else(Vec::new).iter().map(|x| {
                        (artist_map[&x.clone().into_artist_write()], x.clone())
                    }).collect::<HashMap<u32, ArtistWriteContext>>(),
                    write.album.clone().and_then(|x| Some(album_map[&x].clone())),
                ),
                None
            )
        )).collect();

    // fill in existing ones
    let unique_set: HashSet<UniqueTrack> = unique_map.keys().cloned().collect();
    let unique_result_map = get_existing_track_ids(unique_set, &db).await?;
    unique_result_map.into_iter().for_each(|(u, r)| {
        if let Some(id) = r {
            unique_map.get_mut(&u).expect("Uniques from result should be in the originally provided map").1 = Some(id);
        }
    } );

    // create inserts for all remaining ones
    let inserts: HashMap<UniqueTrack, (TrackActiveModel, Vec<TrackArtistActiveModel>)> = unique_map
        .iter()
        .filter(|(_, (_, result))| result.is_none())
        .map(|(unique, ((track, artists, album), result))| {
            let mut model = track_write_to_model(track.clone());
            model.album_id = album.map(|x| Set(Some(x))).unwrap_or(NotSet);
            let track_artist_inserts = artists.into_iter().map(|(id, context)| {
                TrackArtistActiveModel {
                    track_id: NotSet,
                    artist_id: Set(*id),
                    primary: Set(context.primary),
                    artist_alias: Set(context.alias.clone()),
                }
            }).collect();
            (unique.clone(), (model, track_artist_inserts))
        }).collect();

    // not as pretty as batch inserts, but to keep track of which model got which ID (sqlite doesnt guarantee order),
    // we do it all individually
    // using a txn should make the performance cost acceptable with sqlite, and we rarely insert lots of stuff
    // expect for the initial import
    let txn = db.begin().await?;
    let amount_inserts = &inserts.len();
    for (unique, (insert, mut track_artists)) in inserts {
        let db_result = Track::insert(insert).exec(&txn).await?;
        let track_id: u32 = db_result.last_insert_id;
        unique_map.get_mut(&unique)
            .expect("Uniques from insert map should be in the originally provided map")
            .1 = Some(track_id);

        for aux_insert in &mut track_artists {
            aux_insert.track_id = Set(track_id);
        }

        TrackArtist::insert_many(track_artists).exec(&txn).await?;
    }
    txn.commit().await?;

    mark_db_write();
    debug!("Inserted {:?} tracks", amount_inserts);
    Ok(map_to_unique.into_iter().map(|(write, unique)| {
        (write, unique_map[&unique].1.expect("At this point no entities should be unresolved"))
    }).collect())
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
    let txn = db.begin().await?;
    let amount_inserts = &inserts.len();
    for chunk in inserts.iter().collect::<Vec<_>>().chunks(BATCH_SIZE) {
        let chunk_inserts = chunk.to_vec();
        let actual_inserts: Vec<ScrobbleActiveModel> = chunk_inserts.into_iter().map(|(w,am)| am.to_owned()).collect();
        let db_result: Vec<i64> = Scrobble::insert_many(actual_inserts).exec_with_returning_keys(&txn).await?;
        for id in db_result {
            // we basically just get the key to match by input ts even tho we already have that
            // this in the model and could just build a result map based on that
            // this is just for pattern accuracy so we actually confirm each id has a db entry
            let key: ScrobbleWrite = inserts.iter()
                .find(|(w,am)| am.timestamp.clone() == Set(id) ).unwrap().0.clone();
            result.insert(key, id);
        }
    }
    txn.commit().await?;

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

