use axum::extract::Path;
use axum::response::{Html, IntoResponse, Response};
//use dynja::Template;
use askama::Template;
use dynja::minijinja::functions::range;
use crate::database;
use crate::database::views::{ChartsEntry, PerformanceEntry, PulseEntry};
use crate::entity::album::AlbumRead;
use crate::entity::artist::ArtistRead;
use crate::entity::scrobble::ScrobbleRead;
use crate::entity::track::TrackRead;
use crate::timeranges::{RangeType, TimeRange, ALL_TIME};
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

fn get_last_ranges(amount: usize) -> Vec<(RangeType, Vec<TimeRange>)> {
    const RANGE_TYPES: [RangeType; 4] = [RangeType::Year, RangeType::Month, RangeType::Week, RangeType::Day];
    let range_types_and_ranges: Vec<(RangeType, Vec<TimeRange>)> = RANGE_TYPES.into_iter().map(|range_type| {
        // get the last ranges for each type
        let ranges = ALL_TIME.get_subranges(range_type.clone());
        let last12 = ranges[ranges.len().saturating_sub(amount)..].to_owned();
        (range_type, last12)
    } ).collect();

    range_types_and_ranges
}


#[derive(Template)]
#[template(path = "info_artist.html")]
struct ArtistPage {
    artist: ArtistRead,
    track_charts: Vec<ChartsEntry<TrackRead>>,
    scrobbles: Vec<ScrobbleRead>,
    pulses: Vec<(RangeType, Vec<PulseEntry>)>,
    performances: Vec<(RangeType, Vec<PerformanceEntry>)>,
}
pub async fn info_artist(Path(params_path): Path<PathEntity>) -> Response {
    let result = database::repository::artist_info(params_path.id).await.unwrap();
    let tracks = database::repository::charts_tracks(ALL_TIME, Some(result.id), None).await.unwrap();
    let scrobbles = database::repository::scrobbles(ALL_TIME, Some(result.id), None, None, true).await.unwrap();


    let range_types_and_ranges = get_last_ranges(12);
    // async closures unstable
    /*let pulses = range_types_and_ranges.iter().map(async |range_type, ranges| {
        (range_type, database::repository::pulse(ranges.clone(), Some(result.id), None, None).await.unwrap())
    }).collect();
    let performances = range_types_and_ranges.iter().map(async |range_type, ranges| {
        (range_type, database::repository::performance(ranges, Some(result.id), None, None).await.unwrap())
    }).collect();*/
    let mut pulses = vec![];
    for (range_type, ranges) in &range_types_and_ranges {
        pulses.push((range_type.clone(), database::repository::pulse(ranges.clone(), Some(result.id), None, None).await.unwrap()));
    }
    let mut performances = vec![];
    for (range_type, ranges) in &range_types_and_ranges {
        performances.push((range_type.clone(), database::repository::performance(ranges.clone(), Some(result.id), None, None).await.unwrap()));
    }

    let p = ArtistPage {
        artist: result,
        track_charts: tracks,
        scrobbles: scrobbles,
        pulses: pulses,
        performances: performances
    };
    Html(p.render().unwrap()).into_response()
}

#[derive(Template)]
#[template(path = "info_track.html")]
struct TrackPage {
    track: TrackRead,
    scrobbles: Vec<ScrobbleRead>,
    pulses: Vec<(RangeType, Vec<PulseEntry>)>,
    performances: Vec<(RangeType, Vec<PerformanceEntry>)>,
}
pub async fn info_track(Path(params_path): Path<PathEntity>) -> Response {
    let result = database::repository::track_info(params_path.id).await.unwrap();
    let scrobbles = database::repository::scrobbles(ALL_TIME, None, None, Some(result.id), true).await.unwrap();

    let range_types_and_ranges = get_last_ranges(12);
    let mut pulses = vec![];
    for (range_type, ranges) in &range_types_and_ranges {
        pulses.push((range_type.clone(), database::repository::pulse(ranges.clone(), None, None, Some(result.id)).await.unwrap()));
    }
    let mut performances = vec![];
    for (range_type, ranges) in &range_types_and_ranges {
        performances.push((range_type.clone(), database::repository::performance(ranges.clone(), None, None, Some(result.id)).await.unwrap()));
    }

    let p = TrackPage {
        track: result,
        scrobbles: scrobbles,
        pulses: pulses,
        performances: performances
    };
    Html(p.render().unwrap()).into_response()
}

#[derive(Template)]
#[template(path = "info_album.html")]
struct AlbumPage {
    album: AlbumRead,
    track_charts: Vec<ChartsEntry<TrackRead>>,
    scrobbles: Vec<ScrobbleRead>,
    pulses: Vec<(RangeType, Vec<PulseEntry>)>,
    performances: Vec<(RangeType, Vec<PerformanceEntry>)>,
}
pub async fn info_album(Path(params_path): Path<PathEntity>) -> Response {
    let result = database::repository::album_info(params_path.id).await.unwrap();
    let tracks = database::repository::charts_tracks(ALL_TIME, None, Some(result.id)).await.unwrap();
    let scrobbles = database::repository::scrobbles(ALL_TIME, None, Some(result.id), None, true).await.unwrap();

    let range_types_and_ranges = get_last_ranges(12);
    let mut pulses = vec![];
    for (range_type, ranges) in &range_types_and_ranges {
        pulses.push((range_type.clone(), database::repository::pulse(ranges.clone(), None, Some(result.id), None).await.unwrap()));
    }
    let mut performances = vec![];
    for (range_type, ranges) in &range_types_and_ranges {
        performances.push((range_type.clone(), database::repository::performance(ranges.clone(), None, Some(result.id), None).await.unwrap()));
    }

    let p = AlbumPage {
        album: result,
        track_charts: tracks,
        scrobbles: scrobbles,
        pulses: pulses,
        performances: performances
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
