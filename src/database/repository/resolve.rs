use crate::entity::{
    ArtistRead, ArtistModel, Artist, ArtistColumn, ArtistReadContext,
    AlbumRead, AlbumModel, Album, AlbumColumn,
    TrackRead, TrackModel, Track, TrackColumn,
    ScrobbleRead, ScrobbleModel, Scrobble, ScrobbleColumn,
    TrackArtistModel, TrackArtist,
    AlbumArtistModel, AlbumArtist,
};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use std::collections::{HashMap, HashSet};
use crate::database::errors::MalojaError;
use crate::database::repository::{album_model_to_read, artist_model_to_read, scrobble_model_to_read, track_model_to_read};

pub type TrackArtistFull = (TrackArtistModel, ArtistModel);
pub type AlbumArtistFull = (AlbumArtistModel, ArtistModel);
pub type AlbumModelWithRelated = (AlbumModel, HashSet<AlbumArtistFull>);
pub type TrackModelWithRelated = (TrackModel, HashSet<TrackArtistFull>, Option<AlbumModelWithRelated>);
pub type ScrobbleModelWithRelated = (ScrobbleModel, TrackModelWithRelated);


pub async fn resolve_track_ids_to_models(ids: Vec<u32>, db: &DatabaseConnection) -> Result<HashMap<u32, TrackModelWithRelated>, MalojaError> {
    // we can resolve one relation directly with a db call instead of the function
    let db_result: Vec<(TrackModel, Vec<TrackArtistModel>)> = Track::find()
        .filter(TrackColumn::Id.is_in(ids))
        .find_with_related(TrackArtist)
        .all(db)
        .await?;

    let artist_ids: Vec<u32> = db_result.iter().flat_map(|(t, tas)| tas).map(|ta| ta.artist_id).collect();
    let artist_map: HashMap<u32, ArtistModel> = resolve_artist_ids_to_models(artist_ids, &db).await?;

    let album_ids: Vec<u32> = db_result.iter().filter_map(|(t, tas)| t.album_id).collect();
    let album_map: HashMap<u32, AlbumModelWithRelated> = resolve_album_ids_to_models(album_ids, &db).await?;

    let mut result: HashMap<u32, TrackModelWithRelated> = HashMap::new();

    for (track, track_artists) in db_result {
        let album_id = track.album_id.clone();
        result.insert(
            track.id,
            (
                track, // TrackModel
                track_artists.into_iter().map(|ta| {
                    let artist_id = ta.artist_id.clone();
                    (
                        ta, // TrackArtistModel
                        artist_map.get(&artist_id).expect("Artist ID should be in Map").clone() // ArtistModel
                    )
                }).collect(),
                album_id.as_ref().and_then(|x| album_map.get(x)).cloned() // AlbumModelWithRelated
            )
        );
    }

    Ok(result)
}

pub async fn resolve_album_ids_to_models(ids: Vec<u32>, db: &DatabaseConnection) -> Result<HashMap<u32, AlbumModelWithRelated>, MalojaError> {
    let db_result: Vec<(AlbumModel, Vec<AlbumArtistModel>)> = Album::find()
        .filter(AlbumColumn::Id.is_in(ids))
        .find_with_related(AlbumArtist)
        .all(db)
        .await?;

    let artist_ids: Vec<u32> = db_result.iter().flat_map(|(a, aas)| aas).map(|aa| aa.artist_id).collect();
    let artist_map: HashMap<u32, ArtistModel> = resolve_artist_ids_to_models(artist_ids, &db).await?;

    let mut result: HashMap<u32, AlbumModelWithRelated> = HashMap::new();

    for (album, album_artists) in db_result {
        result.insert(
            album.id,
            (
                album, // AlbumModel
                album_artists.into_iter().map(|aa| {
                    let artist_id = aa.artist_id.clone();
                    (
                        aa, // AlbumArtistModel
                        artist_map.get(&artist_id).expect("Artist ID should be in Map").clone() // ArtistModel
                    )
                }).collect(),
            )
        );
    }

    Ok(result)
}

pub async fn resolve_artist_ids_to_models(ids: Vec<u32>, db: &DatabaseConnection) -> Result<HashMap<u32, ArtistModel>, MalojaError> {
    let db_result: Vec<ArtistModel> = Artist::find()
        .filter(ArtistColumn::Id.is_in(ids))
        .all(db)
        .await?;

    let mut result: HashMap<u32, ArtistModel> = HashMap::new();

    for artist in db_result {
        result.insert(
            artist.id,
            artist
        );
    }

    Ok(result)

}

pub async fn resolve_scrobble_ids_to_models(ids: Vec<i64>, db: &DatabaseConnection) -> Result<HashMap<i64, ScrobbleModelWithRelated>, MalojaError> {
    let db_result: Vec<ScrobbleModel> = Scrobble::find()
        .filter(ScrobbleColumn::Timestamp.is_in(ids))
        .all(db)
        .await?;

    // no point getting the track in the same query if we call the other func anyway
    // technically we could save some performance by introducing a weird middle query so that we
    // can get the track here, and then get album and track artists there?

    let track_ids: Vec<u32> = db_result.iter().map(|s| s.track_id).collect();
    let track_map: HashMap<u32, TrackModelWithRelated> = resolve_track_ids_to_models(track_ids, &db).await?;

    let mut result: HashMap<i64, ScrobbleModelWithRelated> = HashMap::new();

    for scrobble in db_result {
        let track_id = scrobble.track_id;
        result.insert(
            scrobble.timestamp,
            (
                scrobble,
                track_map.get(&track_id).expect("Track ID should be in map").clone()
            )
        );
    }

    Ok(result)
}

pub async fn resolve_track_ids(ids: Vec<u32>, db: &DatabaseConnection) -> Result<HashMap<u32, TrackRead>, MalojaError> {
    let models = resolve_track_ids_to_models(ids, db).await?;
    let reads: HashMap<u32, TrackRead> = models.into_iter().map(|(x,y)| (x, track_model_to_read(y))).collect();
    Ok(reads)
}

pub async fn resolve_album_ids(ids: Vec<u32>, db: &DatabaseConnection) -> Result<HashMap<u32, AlbumRead>, MalojaError> {
    let models = resolve_album_ids_to_models(ids, db).await?;
    let reads: HashMap<u32, AlbumRead> = models.into_iter().map(|(x,y)| (x, album_model_to_read(y))).collect();
    Ok(reads)
}

pub async fn resolve_artist_ids(ids: Vec<u32>, db: &DatabaseConnection) -> Result<HashMap<u32, ArtistRead>, MalojaError> {
    let models = resolve_artist_ids_to_models(ids, db).await?;
    let reads: HashMap<u32, ArtistRead> = models.into_iter().map(|(x,y)| (x, artist_model_to_read(y))).collect();
    Ok(reads)
}

pub async fn resolve_scrobble_ids(ids: Vec<i64>, db: &DatabaseConnection) -> Result<HashMap<i64, ScrobbleRead>, MalojaError> {
    let models = resolve_scrobble_ids_to_models(ids, db).await?;
    let reads: HashMap<i64, ScrobbleRead> = models.into_iter().map(|(x,y)| (x, scrobble_model_to_read(y))).collect();
    Ok(reads)
}
