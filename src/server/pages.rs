use axum::extract::Path;
use axum::response::{Html, IntoResponse, Response};
//use dynja::Template;
use askama::Template;
use crate::database;
use crate::database::views::{ChartsEntry, PulseEntry};
use crate::entity::album::AlbumRead;
use crate::entity::artist::ArtistRead;
use crate::entity::scrobble::ScrobbleRead;
use crate::entity::track::TrackRead;
use crate::timeranges::{RangeType, ALL_TIME};
use crate::uri::PathEntity;


/*
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
*/



#[derive(Template)]
#[template(path = "info_artist.html")]
struct ArtistPage {
    artist: ArtistRead,
    track_charts: Vec<ChartsEntry<TrackRead>>,
    scrobbles: Vec<ScrobbleRead>,
    pulse: Vec<PulseEntry>,
}
pub async fn info_artist(Path(params_path): Path<PathEntity>) -> Response {
    let result = database::repository::artist_info(params_path.id).await.unwrap();
    let tracks = database::repository::charts_tracks(ALL_TIME, Some(result.id), None).await.unwrap();
    let scrobbles = database::repository::scrobbles(ALL_TIME, Some(result.id), None, None, true).await.unwrap();
    let rngs = ALL_TIME.get_subranges(RangeType::Year);
    let last12 = rngs[rngs.len().saturating_sub(12)..].to_owned();
    let pulse = database::repository::pulse(last12, Some(result.id), None, None).await.unwrap();
    let p = ArtistPage {
        artist: result,
        track_charts: tracks,
        scrobbles: scrobbles,
        pulse: pulse,
    };
    Html(p.render().unwrap()).into_response()
}

#[derive(Template)]
#[template(path = "info_track.html")]
struct TrackPage {
    track: TrackRead,
    scrobbles: Vec<ScrobbleRead>,
    pulse: Vec<PulseEntry>,
}
pub async fn info_track(Path(params_path): Path<PathEntity>) -> Response {
    let result = database::repository::track_info(params_path.id).await.unwrap();
    let scrobbles = database::repository::scrobbles(ALL_TIME, None, None, Some(result.id), true).await.unwrap();
    let rngs = ALL_TIME.get_subranges(RangeType::Year);
    let last12 = rngs[rngs.len().saturating_sub(12)..].to_owned();
    let pulse = database::repository::pulse(last12, None, None, Some(result.id)).await.unwrap();
    let p = TrackPage {
        track: result,
        scrobbles: scrobbles,
        pulse: pulse,
    };
    Html(p.render().unwrap()).into_response()
}

#[derive(Template)]
#[template(path = "info_album.html")]
struct AlbumPage {
    album: AlbumRead,
    track_charts: Vec<ChartsEntry<TrackRead>>,
    scrobbles: Vec<ScrobbleRead>,
    pulse: Vec<PulseEntry>,
}
pub async fn info_album(Path(params_path): Path<PathEntity>) -> Response {
    let result = database::repository::album_info(params_path.id).await.unwrap();
    let tracks = database::repository::charts_tracks(ALL_TIME, None, Some(result.id)).await.unwrap();
    let scrobbles = database::repository::scrobbles(ALL_TIME, None, Some(result.id), None, true).await.unwrap();
    let rngs = ALL_TIME.get_subranges(RangeType::Year);
    let last12 = rngs[rngs.len().saturating_sub(12)..].to_owned();
    let pulse = database::repository::pulse(last12, None, Some(result.id), None).await.unwrap();
    let p = AlbumPage {
        album: result,
        track_charts: tracks,
        scrobbles: scrobbles,
        pulse: pulse,
    };
    Html(p.render().unwrap()).into_response()
}



#[derive(Template)]
#[template(path = "about.html")]
struct AboutPage {
    version: String,
}
pub async fn about() -> Response {
    let p = AboutPage {
        version: env!("CARGO_PKG_VERSION").to_string(),
    };

    Html(p.render().unwrap()).into_response()
}
