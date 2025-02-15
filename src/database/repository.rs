use std::default::Default;
use std::collections::HashMap;
use log::{debug, info};
use sea_orm::{ColumnTrait, DbErr, EntityTrait, NotSet, QueryFilter};
use sea_orm::ActiveValue::Set;
use crate::database::connect;
use crate::entity;
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

fn normalize(input: &str) -> String {
    input.to_lowercase().replace("_", "-").replace(" ", "-")
}

// alrighty this time we're doing it organized from the start unlike the python monstrosity
// this is totally gonna work this time lmao
// link the relevant xkcd here
#[allow(clippy::collapsible_else_if)]
pub async fn get_or_create_artists(input: Vec<ArtistWrite>) -> HashMap<ArtistWrite, ArtistModel> {
    let db = connect().await;
    let mut result: HashMap<ArtistWrite, Option<ArtistModel>> = HashMap::new();
    input.clone().into_iter().for_each(|artist| {
        result.insert(artist, None);
    });

    // internal ID means instant match - if id supplied but doesn't exist, error!
    // if no ID supplied:
    // - mbid match - use extra table so multiple mbids can match to same artist
    // - spotfiy id match - same
    // - normalized name

    // supplying an ID indicated the client wants to refer to an existing entity - mismatching other info is ignored

    let mut id_map: HashMap<u32, Vec<&ArtistWrite>> = HashMap::new();
    let mut name_map: HashMap<String, Vec<&ArtistWrite>> = HashMap::new();
    for (index, inp) in input.iter().enumerate() {
        if let Some(id) = &inp.id {
            id_map.entry(*id).or_insert(vec![]).push(inp);
        }
        // if we have an ID supplied, we're not gonna use anything else, even if the ID doesn't work - so all other maps in else
        else {
            if let Some(name) = &inp.name {
                name_map.entry(normalize(name)).or_insert(vec![]).push(inp);
            }
        }
    }
    let id_list: Vec<u32> = id_map.keys().cloned().collect();
    let name_list: Vec<String> = name_map.keys().cloned().collect();

    // IDs
    let db_result = Artist::find()
        .filter(ArtistColumn::Id.is_in(id_list))
        .all(&db).await.unwrap();
    for model in db_result {
        let writes = &id_map[&model.id];
        for write in writes {
            result.insert(write.to_owned().clone(), Some(model.clone()));
            // do NOT ask me what the fuck is happening with ownership here i just want it to compile
        }

    }
    // TODO: make sure no supplied IDs are unused - this should be an error instead of just checking for other match methods

    // TODO: mbid and spotify_id

    // Names
    let db_result = Artist::find()
        .filter(ArtistColumn::NameNormalized.is_in(name_list))
        .all(&db).await.unwrap();
    for model in db_result {
        let writes = &name_map[&model.name_normalized];
        for write in writes {
            result.insert(write.to_owned().clone(), Some(model.clone()));
        }
    }

    // All remaining must be created new
    // we dont need any maps here because they will be returned in the order they are supplied
    let mut notfound: Vec<&ArtistWrite> = vec![];
    for (write, opt) in result.iter() {
        if opt.is_none() {
            notfound.push(write);
        }
    }
    if !notfound.is_empty() {
        let inserts: Vec<ArtistActiveModel> = notfound.iter().map(|&x| {
            assert!(x.name.is_some()); //TODO
            let x = x.to_owned();
            ArtistActiveModel {
                id: NotSet,
                name: Set(x.name.clone().unwrap()),
                name_normalized: Set(normalize(&x.name.clone().unwrap())),
                mbid: Set(x.mbid),
                spotify_id: Set(x.spotify_id),
            }
        }).collect();

        //let inserts = inserts.chunks(1).next().unwrap().to_vec();
        let amount_inserts = &inserts.len();
        // this doesnt seem to return all models
        // https://github.com/SeaQL/sea-orm/discussions/2191
        // so for now we just insert and call the whole thing again i guess?
        //let db_result: Vec<ArtistModel> = Artist::insert_many(inserts).exec_with_returning(&db).await.unwrap();
        for chunk in inserts.chunks(BATCH_SIZE) {
            let chunk_inserts = chunk.to_vec();
            let db_result = Artist::insert_many(chunk_inserts).exec(&db).await.unwrap();
        }

        debug!("Inserted {:?} Artists", amount_inserts);
        Box::pin(get_or_create_artists(input)).await
    }
    else {
        let result: HashMap<ArtistWrite, ArtistModel> = result.into_iter().map(|(k,v)| (k,v.expect("This should not happen!").clone())).collect();
        // There should no longer be None variants now
        result
    }
}

#[allow(clippy::collapsible_else_if)]
pub async fn get_or_create_tracks(input: Vec<TrackWrite>) -> HashMap<TrackWrite, TrackModel> {
    let db = connect().await;
    let mut result: HashMap<TrackWrite, Option<TrackModel>> = HashMap::new();
    input.clone().into_iter().for_each(|track| {
        result.insert(track, None);
    });

    // make sure all artists exist
    let artists = input.iter().map(|t| [t.primary_artists.clone().unwrap_or_default(), t.secondary_artists.clone().unwrap_or_default()].concat()).flatten().collect();
    let artist_map = get_or_create_artists(artists).await;

    // make sure all albums exist
    let albums: Vec<AlbumWrite> = input.iter().filter_map(|t| t.album.clone()).collect();
    let album_map = get_or_create_albums(albums).await;


    // as above, but now the name alone isnt enough - we need name and artist exact set match (primary secondary doesnt matter)
    let mut id_map: HashMap<u32, Vec<&TrackWrite>> = HashMap::new();
    let mut title_artists_map: HashMap<(String, Vec<u32>), Vec<&TrackWrite>> = HashMap::new();
    for (index, inp) in input.iter().enumerate() {
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
            let db_result = Track::insert(insert).exec_with_returning(&db).await.unwrap();


            // TODO: MAKE THIS NOT SHIT
            let track_id = db_result.id;

            let track_artist_inserts_primary: Vec<TrackArtistActiveModel> = primary_artists.iter().map(|x| {
                // get the mapped model that definitely has an ID now
                let artist_model = &artist_map[x];
                TrackArtistActiveModel {
                    track_id: Set(track_id),
                    artist_id: Set(artist_model.id),
                    primary: Set(true),
                    artist_alias: Default::default(),
                }
            }).collect();
            let track_artist_inserts_secondary: Vec<TrackArtistActiveModel> = secondary_artists.iter().map(|x| {
                let artist_model = &artist_map[x];
                TrackArtistActiveModel {
                    track_id: Set(track_id),
                    artist_id: Set(artist_model.id),
                    primary: Set(false),
                    artist_alias: Default::default(),
                }
            }).collect();


            if !track_artist_inserts_primary.is_empty() {
                let db_result = TrackArtist::insert_many(track_artist_inserts_primary).exec(&db).await.unwrap();
            }
            if !track_artist_inserts_secondary.is_empty() {
                let db_result = TrackArtist::insert_many(track_artist_inserts_secondary).exec(&db).await.unwrap();
            }


        }

        debug!("Inserted {:?} Tracks", amount_inserts);
        Box::pin(get_or_create_tracks(input)).await
    }
    else {
        let result: HashMap<TrackWrite, TrackModel> = result.into_iter().map(|(k,v)| (k,v.expect("This should not happen!").clone())).collect();
        // There should no longer be None variants now
        result
    }
}



#[allow(clippy::collapsible_else_if)]
pub async fn get_or_create_albums(input: Vec<AlbumWrite>) -> HashMap<AlbumWrite, AlbumModel> {
    let db = connect().await;
    let mut result: HashMap<AlbumWrite, Option<AlbumModel>> = HashMap::new();
    input.clone().into_iter().for_each(|album| {
        result.insert(album, None);
    });

    // make sure all artists exist
    let artists = input.iter().map(|a| a.album_artists.clone().unwrap_or_default()).flatten().collect();
    let artist_map = get_or_create_artists(artists).await;


    // as above, but now the name alone isnt enough - we need name and artist exact set match (primary secondary doesnt matter)
    let mut id_map: HashMap<u32, Vec<&AlbumWrite>> = HashMap::new();
    let mut albumtitle_artists_map: HashMap<(String, Vec<u32>), Vec<&AlbumWrite>> = HashMap::new();
    for (index, inp) in input.iter().enumerate() {
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
            let db_result = Album::insert(insert).exec_with_returning(&db).await.unwrap();


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
                let db_result = AlbumArtist::insert_many(album_artist_inserts).exec(&db).await.unwrap();
            }


        }

        debug!("Inserted {:?} Albums", amount_inserts);
        Box::pin(get_or_create_albums(input)).await
    }
    else {
        let result: HashMap<AlbumWrite, AlbumModel> = result.into_iter().map(|(k,v)| (k,v.expect("This should not happen!").clone())).collect();
        // There should no longer be None variants now
        result
    }
}


#[allow(clippy::collapsible_else_if)]
pub async fn create_scrobbles(input: Vec<ScrobbleWrite>, fail_on_existing: bool) -> HashMap<ScrobbleWrite, ScrobbleModel> {
    // this one is a bit different that the other entity ones because we never supply a scrobblewrite
    // as part of another entity to either create or fetch - scrobbles are only ever created (or patched?)
    let db = connect().await;
    let mut result: HashMap<ScrobbleWrite, Option<ScrobbleModel>> = HashMap::new();
    input.clone().into_iter().for_each(|scrobble| {
        result.insert(scrobble, None);
    });

    // make sure all tracks exist
    let tracks = input.iter().map(|s| s.track.clone()).collect();
    let track_map = get_or_create_tracks(tracks).await;


    // here we have no matching. existing timestamp means existing scrobble, otherwise new
    // normally supplying the ID is a clear indication someone is referring to an existing entity
    // for scrobble, it is feasible to want to submit a new scrobble but use a timestamp that exists
    // so TODO: distinguish between use cases? when do we even refer to existing scrobbles with a write?

    let mut ts_map: HashMap<i64, Vec<&ScrobbleWrite>> = HashMap::new();
    for (index, inp) in input.iter().enumerate() {
        ts_map.entry(inp.timestamp).or_insert(vec![]).push(inp);
    }
    let ts_list: Vec<i64> = ts_map.keys().cloned().collect();

    let db_result = Scrobble::find()
        .filter(ScrobbleColumn::Timestamp.is_in(ts_list))
        .all(&db).await.unwrap();
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
                raw_scrobble: Default::default(),
                origin: Set(x.origin),
                listen_duration: Set(x.listen_duration),
            }
        }).collect();

        let amount_inserts = &inserts.len();

        // for now, insert each one individually so we can actually get the ID
        // i really hope this isnt the permanent solution
        for chunk in inserts.chunks(BATCH_SIZE) {
            let chunk_inserts = chunk.to_vec();
            let db_result = Scrobble::insert_many(chunk_inserts).exec(&db).await.unwrap();
        }

        debug!("Inserted {:?} Scrobbles", amount_inserts);
        Box::pin(create_scrobbles(input, false)).await
    }
    else {
        let result: HashMap<ScrobbleWrite, ScrobbleModel> = result.into_iter().map(|(k,v)| (k,v.expect("This should not happen!").clone())).collect();
        // There should no longer be None variants now
        result
    }
}


pub async fn get_tracks() -> Result<Vec<entity::track::Model>, DbErr> {
    let db = connect().await;
    let tracks = Track::find().all(&db).await?;
    Ok(tracks)
}

pub async fn get_artists() -> Result<Vec<entity::artist::Model>, DbErr> {
    let db = connect().await;
    let artists = Artist::find().all(&db).await?;
    Ok(artists)
}

pub async fn get_albums() -> Result<Vec<entity::album::Model>, DbErr> {
    let db = connect().await;
    let albums = Album::find().all(&db).await?;
    Ok(albums)
}

pub async fn get_scrobbles() -> Result<Vec<entity::scrobble::Model>, DbErr> {
    let db = connect().await;
    let scrobbles = Scrobble::find().all(&db).await?;
    Ok(scrobbles)
}