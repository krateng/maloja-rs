use crate::database::views::{ChartsEntry, PerformanceEntry, PulseEntry};
use crate::entity::album::AlbumRead;
use crate::entity::artist::ArtistRead;
use crate::entity::scrobble::ScrobbleRead;
use crate::entity::track::TrackRead;
use crate::timeranges::{RangeType, ValidTimeRange, ALL_TIME};
use crate::uri::{
    EntityLimit, MalojaQuery, PathEntity, QueryLimitAlbum, QueryLimitArtist, QueryLimitTrack,
    QueryTimerange, QueryTimesteps,
};
use askama::Template;
use axum::extract::{Path, Query, State};
use axum::response::{Html, IntoResponse, Response};
use crate::configuration::app_state::AppState;
use crate::configuration::config::MalojaConfig;

// filters dont work for me
mod filters {
    use askama::Result as AskamaResult;

    pub fn format_in_tz(timestamp: i64) -> AskamaResult<String> {
        // TODO later use config obv
        let tz = chrono_tz::Tz::Europe__Vienna;
        let time = chrono::DateTime::from_timestamp(timestamp, 0).unwrap();
        let fmt = "%Y-%m-%dT%H:%M:%S";
        let local_time = time.with_timezone(&tz);
        Ok(local_time.format(fmt).to_string())
    }
}

const RANGE_TYPES: [RangeType; 4] =
    [RangeType::Year, RangeType::Month, RangeType::Week, RangeType::Day];

fn get_last_ranges(amount: usize) -> Vec<(RangeType, Vec<ValidTimeRange>)> {
    let range_types_and_ranges: Vec<(RangeType, Vec<ValidTimeRange>)> = RANGE_TYPES
        .into_iter()
        .map(|range_type| {
            // get the last ranges for each type
            let ranges = ALL_TIME.get_subranges(range_type.clone());
            let last12 = ranges[ranges.len().saturating_sub(amount)..].to_owned();
            (range_type, last12)
        })
        .collect();

    range_types_and_ranges
}

fn get_preset_ranges() -> Vec<ValidTimeRange> {
    let mut result: Vec<ValidTimeRange> =
        RANGE_TYPES.into_iter().map(|range_type| range_type.get_current()).collect();
    result.insert(0, ALL_TIME);
    result
}

#[derive(Template)]
#[template(path = "info_artist.html")]
struct ArtistPage {
    config: MalojaConfig,
    artist: ArtistRead,
    entity_limit: EntityLimit, // we always pass this wrapper to all pages for potential includes
    track_charts: Vec<ChartsEntry<TrackRead>>,
    scrobbles: Vec<ScrobbleRead>,
    pulses: Vec<(RangeType, Vec<PulseEntry>)>,
    performances: Vec<(RangeType, Vec<PerformanceEntry>)>,
}
pub async fn info_artist(
    State(state): State<AppState>,
    Path(params_path): Path<PathEntity>
) -> Response {
    let result = state.db.artist_info(params_path.id).await.unwrap();
    let tracks =
        state.db.charts_tracks(&ALL_TIME, Some(result.id), None).await.unwrap();
    let scrobbles =
        state.db.scrobbles(&ALL_TIME, Some(result.id), None, None, true).await.unwrap();

    let range_types_and_ranges = get_last_ranges(12);
    // async closures unstable
    /*let pulses = range_types_and_ranges.iter().map(async |range_type, ranges| {
        (range_type, state.db.pulse(ranges.clone(), Some(result.id), None, None).await.unwrap())
    }).collect();
    let performances = range_types_and_ranges.iter().map(async |range_type, ranges| {
        (range_type, state.db.performance(ranges, Some(result.id), None, None).await.unwrap())
    }).collect();*/
    let mut pulses = vec![];
    for (range_type, ranges) in &range_types_and_ranges {
        pulses.push((
            range_type.clone(),
            state.db.pulse(ranges.clone(), Some(result.id), None, None).await.unwrap(),
        ));
    }
    let mut performances = vec![];
    for (range_type, ranges) in &range_types_and_ranges {
        performances.push((
            range_type.clone(),
            state.db.performance(ranges.clone(), Some(result.id), None, None)
                .await
                .unwrap(),
        ));
    }

    let p = ArtistPage {
        config: state.config,
        artist: result.clone(),
        entity_limit: EntityLimit { artist: Some(result), track: None, album: None },
        track_charts: tracks,
        scrobbles: scrobbles,
        pulses: pulses,
        performances: performances,
    };
    Html(p.render().unwrap()).into_response()
}

#[derive(Template)]
#[template(path = "info_track.html")]
struct TrackPage {
    config: MalojaConfig,
    track: TrackRead,
    entity_limit: EntityLimit, // we always pass this wrapper to all pages for potential includes
    scrobbles: Vec<ScrobbleRead>,
    pulses: Vec<(RangeType, Vec<PulseEntry>)>,
    performances: Vec<(RangeType, Vec<PerformanceEntry>)>,
}
pub async fn info_track(
    State(state): State<AppState>,
    Path(params_path): Path<PathEntity>
) -> Response {
    let result = state.db.track_info(params_path.id).await.unwrap();
    let scrobbles =
        state.db.scrobbles(&ALL_TIME, None, None, Some(result.id), true).await.unwrap();

    let range_types_and_ranges = get_last_ranges(12);
    let mut pulses = vec![];
    for (range_type, ranges) in &range_types_and_ranges {
        pulses.push((
            range_type.clone(),
            state.db.pulse(ranges.clone(), None, None, Some(result.id)).await.unwrap(),
        ));
    }
    let mut performances = vec![];
    for (range_type, ranges) in &range_types_and_ranges {
        performances.push((
            range_type.clone(),
            state.db.performance(ranges.clone(), None, None, Some(result.id))
                .await
                .unwrap(),
        ));
    }

    let p = TrackPage {
        config: state.config,
        track: result.clone(),
        entity_limit: EntityLimit { track: Some(result), album: None, artist: None },
        scrobbles: scrobbles,
        pulses: pulses,
        performances: performances,
    };
    Html(p.render().unwrap()).into_response()
}

#[derive(Template)]
#[template(path = "info_album.html")]
struct AlbumPage {
    config: MalojaConfig,
    album: AlbumRead,
    entity_limit: EntityLimit, // we always pass this wrapper to all pages for potential includes
    track_charts: Vec<ChartsEntry<TrackRead>>,
    scrobbles: Vec<ScrobbleRead>,
    pulses: Vec<(RangeType, Vec<PulseEntry>)>,
    performances: Vec<(RangeType, Vec<PerformanceEntry>)>,
}
pub async fn info_album(
    State(state): State<AppState>,
    Path(params_path): Path<PathEntity>
) -> Response {
    let result = state.db.album_info(params_path.id).await.unwrap();
    let tracks =
        state.db.charts_tracks(&ALL_TIME, None, Some(result.id)).await.unwrap();
    let scrobbles =
        state.db.scrobbles(&ALL_TIME, None, Some(result.id), None, true).await.unwrap();

    let range_types_and_ranges = get_last_ranges(12);
    let mut pulses = vec![];
    for (range_type, ranges) in &range_types_and_ranges {
        pulses.push((
            range_type.clone(),
            state.db.pulse(ranges.clone(), None, Some(result.id), None).await.unwrap(),
        ));
    }
    let mut performances = vec![];
    for (range_type, ranges) in &range_types_and_ranges {
        performances.push((
            range_type.clone(),
            state.db.performance(ranges.clone(), None, Some(result.id), None)
                .await
                .unwrap(),
        ));
    }

    let p = AlbumPage {
        config: state.config,
        album: result.clone(),
        entity_limit: EntityLimit { album: Some(result), track: None, artist: None },
        track_charts: tracks,
        scrobbles: scrobbles,
        pulses: pulses,
        performances: performances,
    };
    Html(p.render().unwrap()).into_response()
}

#[derive(Template)]
#[template(path = "pulse.html")]
struct PulsePage {
    config: MalojaConfig,
    pulse: Vec<PulseEntry>,
    timerange: ValidTimeRange,
    timestep: RangeType,
    entity_limit: EntityLimit,
    preset_steps: Vec<RangeType>,
    preset_ranges: Vec<ValidTimeRange>,
}
pub async fn pulse(
    State(state): State<AppState>,
    Query(params_timesteps): Query<QueryTimesteps>,
    Query(params_artist): Query<QueryLimitArtist>,
    Query(params_album): Query<QueryLimitAlbum>,
    Query(params_track): Query<QueryLimitTrack>,
    Query(params_timerange): Query<QueryTimerange>,
) -> Response {
    let timerange = params_timerange.to_timerange().unwrap();
    let timestep = params_timesteps.to_rangetype().unwrap();
    let subranges = timerange.get_subranges(timestep.clone());
    let pulse = state.db.pulse(
        subranges,
        params_artist.to_artist_id(),
        params_album.to_album_id(),
        params_track.to_track_id(),
    )
    .await
    .unwrap();
    // TODO we probably want to restrict this and only allow one entity limit

    let (mut artist, mut track, mut album) = (None, None, None);
    if let Some(artist_id) = params_artist.to_artist_id() {
        artist = Some(state.db.artist_info(artist_id).await.unwrap());
    }
    if let Some(track_id) = params_track.to_track_id() {
        track = Some(state.db.track_info(track_id).await.unwrap());
    }
    if let Some(album_id) = params_album.to_album_id() {
        album = Some(state.db.album_info(album_id).await.unwrap());
    }

    let p = PulsePage {
        config: state.config,
        pulse: pulse,
        timerange: timerange,
        timestep: timestep,
        entity_limit: EntityLimit { artist, album, track },
        preset_steps: RANGE_TYPES.to_vec(),
        preset_ranges: get_preset_ranges(),
    };
    Html(p.render().unwrap()).into_response()
}

#[derive(Template)]
#[template(path = "performance.html")]
struct PerformancePage {
    config: MalojaConfig,
    performance: Vec<PerformanceEntry>,
    timerange: ValidTimeRange,
    timestep: RangeType,
    entity_limit: EntityLimit,
    preset_steps: Vec<RangeType>,
    preset_ranges: Vec<ValidTimeRange>,
}
pub async fn performance(
    State(state): State<AppState>,
    Query(params_timesteps): Query<QueryTimesteps>,
    Query(params_artist): Query<QueryLimitArtist>,
    Query(params_album): Query<QueryLimitAlbum>,
    Query(params_track): Query<QueryLimitTrack>,
    Query(params_timerange): Query<QueryTimerange>,
) -> Response {
    let timerange = params_timerange.to_timerange().unwrap();
    let timestep = params_timesteps.to_rangetype().unwrap();
    let subranges = timerange.get_subranges(timestep.clone());
    let performance = state.db.performance(
        subranges,
        params_artist.to_artist_id(),
        params_album.to_album_id(),
        params_track.to_track_id(),
    )
    .await
    .unwrap();
    // TODO we probably want to restrict this and only allow one entity limit

    let (mut artist, mut track, mut album) = (None, None, None);
    if let Some(artist_id) = params_artist.to_artist_id() {
        artist = Some(state.db.artist_info(artist_id).await.unwrap());
    }
    if let Some(track_id) = params_track.to_track_id() {
        track = Some(state.db.track_info(track_id).await.unwrap());
    }
    if let Some(album_id) = params_album.to_album_id() {
        album = Some(state.db.album_info(album_id).await.unwrap());
    }

    let p = PerformancePage {
        config: state.config,
        performance: performance,
        timerange: timerange,
        timestep: timestep,
        entity_limit: EntityLimit { artist, album, track },
        preset_steps: RANGE_TYPES.to_vec(),
        preset_ranges: get_preset_ranges(),
    };
    Html(p.render().unwrap()).into_response()
}

#[derive(Template)]
#[template(path = "scrobbles.html")]
struct ScrobblesPage {
    config: MalojaConfig,
    scrobbles: Vec<ScrobbleRead>,
    entity_limit: EntityLimit,
    timerange: ValidTimeRange,
    preset_ranges: Vec<ValidTimeRange>,
}
pub async fn scrobbles(
    State(state): State<AppState>,
    Query(params_artist): Query<QueryLimitArtist>,
    Query(params_album): Query<QueryLimitAlbum>,
    Query(params_track): Query<QueryLimitTrack>,
    Query(params_timerange): Query<QueryTimerange>,
) -> Response {
    let timerange = params_timerange.to_timerange().unwrap();
    let scrobbles = state.db.scrobbles(
        &timerange,
        params_artist.to_artist_id(),
        params_album.to_album_id(),
        params_track.to_track_id(),
        true,
    )
    .await
    .unwrap();

    let (mut artist, mut track, mut album) = (None, None, None);
    if let Some(artist_id) = params_artist.to_artist_id() {
        artist = Some(state.db.artist_info(artist_id).await.unwrap());
    }
    if let Some(track_id) = params_track.to_track_id() {
        track = Some(state.db.track_info(track_id).await.unwrap());
    }
    if let Some(album_id) = params_album.to_album_id() {
        album = Some(state.db.album_info(album_id).await.unwrap());
    }

    let p = ScrobblesPage {
        config: state.config,
        scrobbles: scrobbles,
        entity_limit: EntityLimit { artist, album, track },
        timerange,
        preset_ranges: get_preset_ranges(),
    };
    Html(p.render().unwrap()).into_response()
}

#[derive(Template)]
#[template(path = "about.html")]
struct AboutPage {
    version: String,
}
pub async fn about() -> Response {
    let p = AboutPage { version: env!("CARGO_PKG_VERSION").to_string() };

    Html(p.render().unwrap()).into_response()
}
