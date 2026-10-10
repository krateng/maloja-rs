use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use log::debug;
use sea_orm::{ColumnTrait, EntityTrait, NotSet, QueryFilter};
use sea_orm::ActiveValue::Set;
use crate::database::{connect, mark_db_write, normalize};
use crate::database::errors::MalojaError;
use crate::entity::{
    album::{Entity as Album, Model as AlbumModel, ActiveModel as AlbumActiveModel, Column as AlbumColumn, AlbumWrite, AlbumRead},
    track::{Entity as Track, Model as TrackModel, ActiveModel as TrackActiveModel, Column as TrackColumn, TrackWrite, TrackRead},
    artist::{Entity as Artist, Model as ArtistModel, ActiveModel as ArtistActiveModel, Column as ArtistColumn, ArtistWrite, ArtistRead},
    scrobble::{Entity as Scrobble, Model as ScrobbleModel, ActiveModel as ScrobbleActiveModel, Column as ScrobbleColumn, ScrobbleWrite},
    track_artist::{Entity as TrackArtist, ActiveModel as TrackArtistActiveModel},
    album_artist::{Entity as AlbumArtist, ActiveModel as AlbumArtistActiveModel},
};

/// How many entities should be inserted into the Database in one go
const BATCH_SIZE: usize = 250;



/// Object that maps a number of incoming [ScrobbleWrite]s to the resulting [ScrobbleModel] that will actually be used.
type ScrobbleWriteMap = HashMap<ScrobbleWrite, ScrobbleModel>;
/// Object that maps a number of incoming [TrackWrite]s to the resulting [TrackModel] that will actually be used.
type TrackWriteMap = HashMap<TrackWrite, TrackModel>;
/// Object that maps a number of incoming [AlbumWrite]s to the resulting [AlbumModel] that will actually be used.
type AlbumWriteMap = HashMap<AlbumWrite, AlbumModel>;
/// Object that maps a number of incoming [ArtistWrite]s to the resulting [ArtistModel] that will actually be used.
type ArtistWriteMap = HashMap<ArtistWrite, Arc<ArtistModel>>;

// alrighty this time we're doing it organized from the start unlike the python monstrosity
// this is totally gonna work this time lmao
// link the relevant xkcd here
#[allow(clippy::collapsible_else_if)]
pub async fn get_or_create_artists(input: Vec<ArtistWrite>) -> Result<ArtistWriteMap, MalojaError> {
    let db = connect().await?;

    // our final result, to be filled as we get the models
    let mut result: ArtistWriteMap = HashMap::new();
    // all currently unmatched writes, owned by this structure until they are moved to the result map
    let mut todo: HashSet<ArtistWrite> = input.iter().cloned().collect();

    // collect all entries that we already know will be duplicates based on their input fields
    // intermediately own them, so we always just move the owned writes around
    // and never clone them
    let mut todo_by_id: HashMap<u32, HashSet<ArtistWrite>> = HashMap::new();
    let mut todo_by_normalized_name: HashMap<String, HashSet<ArtistWrite>> = HashMap::new();

    // fill our maps with data
    let mut todo_remaining: HashSet<ArtistWrite> = HashSet::new();
    for inp in todo.drain() {
        if let Some(id) = inp.id {
            todo_by_id.entry(id).or_insert_with(HashSet::new).insert(inp);
        }
        // if we have an ID supplied, we're not gonna use anything else, even if the ID doesn't work - so all other maps in else
        else if let Some(name) = &inp.name {
            todo_by_normalized_name.entry(normalize(&*name)).or_insert_with(HashSet::new).insert(inp);
        }
        // TODO: mbid and spotify_id
        else {
            todo_remaining.insert(inp);
        }
    }
    todo = todo_remaining;
    assert!(todo.is_empty());

    // DB query based on IDs
    let db_result: Vec<ArtistModel> = Artist::find()
        .filter(ArtistColumn::Id.is_in(todo_by_id.keys()))
        .all(&db).await?;
    // for each found entry, now fetch the writes associated with this ID and add the mapping to the result
    for model in db_result {
        let arc = Arc::new(model);
        for write in todo_by_id.remove(&arc.id).expect("ID Map does not contain returned ID even though we fetched from DB with that ID Map") {
            result.insert(
                write,
                Arc::clone(&arc)
            );
        }
    }
    assert!(todo_by_id.is_empty());
    // TODO: make sure no supplied IDs are unused - this should be an error

    // DB query based on normalized names
    let db_result: Vec<ArtistModel> = Artist::find()
        .filter(ArtistColumn::NameNormalized.is_in(todo_by_normalized_name.keys()))
        .all(&db).await?;
    // for each found entry, now fetch the writes associated with this name and add the mapping to the result
    for model in db_result {
        let arc = Arc::new(model);
        for write in todo_by_normalized_name.remove(&arc.name_normalized).expect("Name Map does not contain returned name even though we fetched from DB with that name Map") {
            result.insert(
                write,
                Arc::clone(&arc)
            );
        }
    }
    // the rest stays in the name map for now so we can do inserts
    if !todo_by_normalized_name.is_empty() {
        // All remaining must be created new

        let inserts: Vec<ArtistActiveModel> = todo_by_normalized_name.iter().map(|(x, y)| {
            let representative_write = y.iter().next().expect("We created an empty write list for a normalized name");
            ArtistActiveModel {
                id: NotSet,
                name: Set(representative_write.name.clone().expect("All newly to write artists should have a name")),
                name_normalized: Set(x.clone()),
                mbid: Set(representative_write.mbid.clone()),
                spotify_id: Set(representative_write.spotify_id.clone()),
            }
        }).collect();

        let amount_inserts = &inserts.len();

        for chunk in inserts.chunks(BATCH_SIZE) {
            let chunk_inserts = chunk.to_vec();
            let db_result: Vec<ArtistModel> = Artist::insert_many(chunk_inserts).exec_with_returning(&db).await?;
            for model in db_result {
                let arc = Arc::new(model);
                for write in todo_by_normalized_name.remove(&arc.name_normalized).expect("Name Map does not contain returned name even though we inserted into DB with that name Map") {
                    result.insert(
                        write,
                        Arc::clone(&arc)
                    );
                }
            }
        }

        mark_db_write();
        debug!("Inserted {:?} Artists", amount_inserts);
    }

    assert!(todo_by_normalized_name.is_empty());
    assert!(input.iter().all(|x| result.contains_key(x)));
    Ok(result)

}

#[allow(clippy::collapsible_else_if)]
pub async fn get_or_create_tracks(input: Vec<TrackWrite>) -> Result<TrackWriteMap, MalojaError> {
    let db = connect().await?;
    let mut result: HashMap<TrackWrite, Option<TrackModel>> = HashMap::new();
    input.clone().into_iter().for_each(|track| {
        result.insert(track, None);
    });

    // make sure all artists exist
    let artists: Vec<ArtistWrite> = input.iter().flat_map(
        |t: &TrackWrite| [t.primary_artists.clone().unwrap_or_default(), t.secondary_artists.clone().unwrap_or_default()].concat()
    ).collect();
    let artist_map = get_or_create_artists(artists).await?;

    // make sure all albums exist
    let albums: Vec<AlbumWrite> = input.iter().filter_map(
        |t: &TrackWrite| t.album.clone()
    ).collect();
    let album_map = get_or_create_albums(albums).await?;


    // as above, but now the name alone isnt enough - we need name and artist exact set match (primary secondary doesnt matter)
    let mut id_map: HashMap<u32, Vec<&TrackWrite>> = HashMap::new();
    let mut title_artists_map: HashMap<(String, Vec<u32>), Vec<&TrackWrite>> = HashMap::new();
    for (_index, inp) in input.iter().enumerate() {
        if let Some(id) = &inp.id {
            id_map.entry(*id).or_insert(vec![]).push(inp);
        }
        // if we have an ID supplied, we're not gonna use anything else, even if the ID doesn't work - so all other maps in else
        else {
            if let Some(title) = &inp.title {
                // artists should now all exist so we can get IDs
                let artists = [inp.to_owned().primary_artists.unwrap_or_default(), inp.to_owned().secondary_artists.unwrap_or_default()]
                    .concat();
                let mut artist_ids = artists.into_iter().map(|x| artist_map[&x].id).collect::<Vec<u32>>();
                artist_ids.sort();
                title_artists_map.entry((normalize(title),artist_ids)).or_insert(vec![]).push(inp);
            }
        }
    }
    let id_list: Vec<u32> = id_map.keys().cloned().collect();
    let title_artists_list: Vec<(String, Vec<u32>)> = title_artists_map.keys().cloned().collect();

    // IDs
    let db_result = Track::find()
        .filter(TrackColumn::Id.is_in(id_list))
        .all(&db).await.unwrap();
    for model in db_result {
        let writes = &id_map[&model.id];
        for write in writes {
            result.insert(write.to_owned().clone(), Some(model.clone()));
        }

    }
    // TODO: make sure no supplied IDs are unused - this should be an error instead of just checking for other match methods

    // TODO: mbid and spotify_id

    // Titles + Artists
    // we'll just ask the database for the matching titles to avoid some crazy super query.
    // matching titles with different artists are already gonna be rare, we can just check in code after
    let title_list: Vec<String> = title_artists_list.into_iter().map(|x| x.0).collect();
    let db_result = Track::find()
        .filter(TrackColumn::TitleNormalized.is_in(title_list))
        //.join(JoinType::LeftJoin, TrackRelation::TrackArtist.def())
        //.select_also(TrackArtistEntity)
        .find_with_related(Artist)
        .all(&db).await.unwrap();
    for (track_model, artist_models) in db_result {
        let mut artist_ids: Vec<u32> = artist_models.iter().map(|x| x.id).collect();
        artist_ids.sort();
        let potential_key = (track_model.title_normalized.clone(), artist_ids);
        if title_artists_map.contains_key(&potential_key) {
            let writes = &title_artists_map[&potential_key];
            for write in writes {
                result.insert(write.to_owned().clone(), Some(track_model.clone()));
            }
        }
    }
    // wtf am i even writing


    // All remaining must be created new
    // we dont need any maps here because they will be returned in the order they are supplied
    let mut notfound: Vec<&TrackWrite> = vec![];
    for (write, opt) in result.iter() {
        if opt.is_none() {
            notfound.push(write);
        }
    }
    if !notfound.is_empty() {
        let inserts: Vec<(TrackActiveModel, Vec<ArtistWrite>, Vec<ArtistWrite>)> = notfound.iter().map(|&x| {
            assert!(x.title.is_some());
            // TODO: do we enforce artists?
            let x = x.to_owned();

            (TrackActiveModel {
                id: NotSet,
                title: Set(x.title.clone().unwrap()),
                title_normalized: Set(normalize(&x.title.clone().unwrap())),
                track_length: Set(x.track_length),
                album_id: if let Some(album) = x.album { Set(Some(album_map.get(&album).unwrap().id)) } else { NotSet },
                mbid: Set(x.mbid.clone()),
                spotify_id: Set(x.spotify_id.clone()),
            },
             x.primary_artists.unwrap_or_default(),
             x.secondary_artists.unwrap_or_default())
        }).collect();

        let amount_inserts = &inserts.len();

        // for now, insert each one individually so we can actually get the ID
        // i really hope this isnt the permanent solution
        for (insert, primary_artists, secondary_artists) in inserts {
            let db_result = Track::insert(insert).exec_with_returning(&db).await?;


            // TODO: MAKE THIS NOT SHIT
            let track_id = db_result.id;

            let mut already_encountered_artist_ids = HashSet::new();

            let primary = primary_artists.iter().map(|a| (a, true));
            let secondary = secondary_artists.iter().map(|a| (a, false));
            let inserts: Vec<TrackArtistActiveModel> = primary.chain(secondary).filter_map(|(a, primary)| {
                // get the mapped model that definitely has an ID now
                let artist_model = &artist_map[a];
                // .then only fires if we inserted
                already_encountered_artist_ids.insert(artist_model.id).then(|| TrackArtistActiveModel {
                    track_id: Set(track_id),
                    artist_id: Set(artist_model.id),
                    primary: Set(primary),
                    artist_alias: Default::default(),
                })
            }).collect();

            if !inserts.is_empty() {
                let db_result = TrackArtist::insert_many(inserts).exec(&db).await?;
            }

        }

        mark_db_write();

        debug!("Inserted {:?} Tracks", amount_inserts);
        Box::pin(get_or_create_tracks(input)).await
    }
    else {
        let result: TrackWriteMap = result.into_iter().map(|(k,v)| (k, v.expect("This should not happen!").clone())).collect();
        // There should no longer be None variants now
        Ok(result)
    }
}



#[allow(clippy::collapsible_else_if)]
pub async fn get_or_create_albums(input: Vec<AlbumWrite>) -> Result<AlbumWriteMap, MalojaError> {
    let db = connect().await?;
    let mut result: HashMap<AlbumWrite, Option<AlbumModel>> = HashMap::new();
    input.clone().into_iter().for_each(|album| {
        result.insert(album, None);
    });

    // make sure all artists exist
    let artists = input.iter().map(|a| a.album_artists.clone().unwrap_or_default()).flatten().collect();
    let artist_map = get_or_create_artists(artists).await?;


    // as above, but now the name alone isnt enough - we need name and artist exact set match (primary secondary doesnt matter)
    let mut id_map: HashMap<u32, Vec<&AlbumWrite>> = HashMap::new();
    let mut albumtitle_artists_map: HashMap<(String, Vec<u32>), Vec<&AlbumWrite>> = HashMap::new();
    for (_index, inp) in input.iter().enumerate() {
        if let Some(id) = &inp.id {
            id_map.entry(*id).or_insert(vec![]).push(inp);
        }
        // if we have an ID supplied, we're not gonna use anything else, even if the ID doesn't work - so all other maps in else
        else {
            if let Some(title) = &inp.album_title {
                // artists should now all exist so we can get IDs
                let artists = inp.to_owned().album_artists.unwrap_or_default();
                let mut artist_ids = artists.into_iter().map(|x| artist_map[&x].id).collect::<Vec<u32>>();
                artist_ids.sort();
                albumtitle_artists_map.entry((normalize(title),artist_ids)).or_insert(vec![]).push(inp);
            }
        }
    }
    let id_list: Vec<u32> = id_map.keys().cloned().collect();
    let albumtitle_artists_list: Vec<(String, Vec<u32>)> = albumtitle_artists_map.keys().cloned().collect();

    // IDs
    let db_result = Album::find()
        .filter(AlbumColumn::Id.is_in(id_list))
        .all(&db).await.unwrap();
    for model in db_result {
        let writes = &id_map[&model.id];
        for write in writes {
            result.insert(write.to_owned().clone(), Some(model.clone()));
        }

    }
    // TODO: make sure no supplied IDs are unused - this should be an error instead of just checking for other match methods

    // TODO: mbid and spotify_id

    // Album Titles + Album Artists
    // we'll just ask the database for the matching titles to avoid some crazy super query.
    // matching titles with different artists are already gonna be rare, we can just check in code after
    let title_list: Vec<String> = albumtitle_artists_list.into_iter().map(|x| x.0).collect();
    let db_result = Album::find()
        .filter(AlbumColumn::AlbumTitleNormalized.is_in(title_list))
        //.join(JoinType::LeftJoin, TrackRelation::TrackArtist.def())
        //.select_also(TrackArtistEntity)
        .find_with_related(Artist)
        .all(&db).await.unwrap();
    for (album_model, artist_models) in db_result {
        let mut artist_ids: Vec<u32> = artist_models.iter().map(|x| x.id).collect();
        artist_ids.sort();
        let potential_key = (album_model.album_title_normalized.clone(), artist_ids);
        if albumtitle_artists_map.contains_key(&potential_key) {
            let writes = &albumtitle_artists_map[&potential_key];
            for write in writes {
                result.insert(write.to_owned().clone(), Some(album_model.clone()));
            }
        }
    }

    // All remaining must be created new
    // we dont need any maps here because they will be returned in the order they are supplied
    let mut notfound: Vec<&AlbumWrite> = vec![];
    for (write, opt) in result.iter() {
        if opt.is_none() {
            notfound.push(write);
        }
    }
    if !notfound.is_empty() {
        let inserts: Vec<(AlbumActiveModel, Vec<ArtistWrite>)> = notfound.iter().map(|&x| {
            assert!(x.album_title.is_some());
            // TODO: do we enforce artists?
            let x = x.to_owned();

            (AlbumActiveModel {
                id: NotSet,
                album_title: Set(x.album_title.clone().unwrap()),
                album_title_normalized: Set(normalize(&x.album_title.clone().unwrap())),
                mbid: Set(x.mbid),
                spotify_id: Set(x.spotify_id),
            },
             x.album_artists.unwrap_or_default())
        }).collect();

        let amount_inserts = &inserts.len();

        // for now, insert each one individually so we can actually get the ID
        // i really hope this isnt the permanent solution
        for (insert, artists) in inserts {
            let db_result = Album::insert(insert).exec_with_returning(&db).await?;


            // TODO: MAKE THIS NOT SHIT
            let album_id = db_result.id;

            let album_artist_inserts: Vec<AlbumArtistActiveModel> = artists.iter().map(|x| {
                // get the mapped model that definitely has an ID now
                let artist_model = &artist_map[x];
                AlbumArtistActiveModel {
                    album_id: Set(album_id),
                    artist_id: Set(artist_model.id),
                }
            }).collect();


            if !album_artist_inserts.is_empty() {
                let db_result = AlbumArtist::insert_many(album_artist_inserts).exec(&db).await?;
            }


        }

        mark_db_write();

        debug!("Inserted {:?} Albums", amount_inserts);
        Box::pin(get_or_create_albums(input)).await
    }
    else {
        let result: AlbumWriteMap = result.into_iter().map(|(k,v)| (k, v.expect("This should not happen!").clone())).collect();
        // There should no longer be None variants now
        Ok(result)
    }
}


#[allow(clippy::collapsible_else_if)]
pub async fn create_scrobbles(input: Vec<ScrobbleWrite>, fail_on_existing: bool) -> Result<ScrobbleWriteMap, MalojaError> {
    // this one is a bit different that the other entity ones because we never supply a scrobblewrite
    // as part of another entity to either create or fetch - scrobbles are only ever created (or patched?)
    let db = connect().await?;
    let mut result: HashMap<ScrobbleWrite, Option<ScrobbleModel>> = HashMap::new();
    input.clone().into_iter().for_each(|scrobble| {
        result.insert(scrobble, None);
    });

    // make sure all tracks exist
    let tracks = input.iter().map(|s| s.track.clone()).collect();
    let track_map = get_or_create_tracks(tracks).await?;


    // here we have no matching. existing timestamp means existing scrobble, otherwise new
    // normally supplying the ID is a clear indication someone is referring to an existing entity
    // for scrobble, it is feasible to want to submit a new scrobble but use a timestamp that exists
    // so TODO: distinguish between use cases? when do we even refer to existing scrobbles with a write?

    let mut ts_map: HashMap<i64, Vec<&ScrobbleWrite>> = HashMap::new();
    for (_index, inp) in input.iter().enumerate() {
        ts_map.entry(inp.timestamp).or_insert(vec![]).push(inp);
    }
    let ts_list: Vec<i64> = ts_map.keys().cloned().collect();

    let db_result = Scrobble::find()
        .filter(ScrobbleColumn::Timestamp.is_in(ts_list))
        .all(&db).await?;
    for model in db_result {
        let writes = &ts_map[&model.timestamp];
        for write in writes {
            result.insert(write.to_owned().clone(), Some(model.clone()));
        }

    }

    // TODO error on found


    let mut notfound: Vec<&ScrobbleWrite> = vec![];
    for (write, opt) in result.iter() {
        if opt.is_none() {
            notfound.push(write);
        }
    }
    if !notfound.is_empty() {
        let inserts: Vec<ScrobbleActiveModel> = notfound.iter().map(|&x| {
            let x = x.to_owned();

            ScrobbleActiveModel {
                timestamp: Set(x.timestamp),
                track_id: Set(track_map[&x.track].id),
                raw_scrobble: Set("{}".into()),
                origin: Set(x.origin),
                listen_duration: Set(x.listen_duration),
            }
        }).collect();

        let amount_inserts = &inserts.len();

        // for now, insert each one individually so we can actually get the ID
        // i really hope this isnt the permanent solution
        for chunk in inserts.chunks(BATCH_SIZE) {
            let chunk_inserts = chunk.to_vec();
            let db_result = Scrobble::insert_many(chunk_inserts).exec(&db).await?;
        }

        mark_db_write();

        debug!("Inserted {:?} Scrobbles", amount_inserts);
        Box::pin(create_scrobbles(input, false)).await
    }
    else {
        let result: ScrobbleWriteMap = result.into_iter().map(|(k,v)| (k, v.expect("This should not happen!").clone())).collect();
        // There should no longer be None variants now
        Ok(result)
    }
}