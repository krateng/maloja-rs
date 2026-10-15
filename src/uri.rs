use crate::database::errors::MalojaError;
use crate::database::views::{Paginated, PaginationInfo};
use crate::entity::album::AlbumRead;
use crate::entity::artist::ArtistRead;
use crate::entity::track::TrackRead;
use crate::timeranges::{RangeType, ValidTimeRange, ALL_TIME};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::cmp::min;
use std::collections::HashMap;
use std::convert::identity;
use sea_query::Iden;
use utoipa::IntoParams;

// Every category of query / path arguments should have struct that converts both to and from the backend object that we use to represent its meaning
// each struct also implements a function to convert to the raw query string so we can chain those in templates

pub trait MalojaQuery {
    fn to_querystring(&self) -> String
    where
        Self: Serialize,
    {
        serde_urlencoded::to_string(self).unwrap()
    }
}

// Query args
#[derive(Deserialize, IntoParams, Debug)]
#[into_params(parameter_in=Query)]
pub struct QueryPagination {
    page: Option<u32>,
    per_page: Option<u32>,
}
impl QueryPagination {
    pub fn paginate_results<T: Clone>(&self, results: Vec<T>) -> Paginated<T> {
        let page = self.page.unwrap_or(1);
        let per_page = self.per_page.unwrap_or(50);
        let start_index = ((page - 1) * per_page) as usize;
        let end_index = start_index + per_page as usize ;
        let end_index = min(end_index, results.len());
        // avoid errors if we request non existent pages - just show empty
        let start_index = min(start_index, end_index);
        let results_slice = results[start_index..end_index].to_owned();
        let pages = (results.len() + (per_page as usize) - 1) / (per_page as usize); // Division that rounds up
        Paginated {
            pagination: PaginationInfo {
                page: page,
                pages: pages as u32,
                items_per_page: per_page,
                items_total: results.len() as u32,
            },
            result: results_slice,
        }
    }
}

#[derive(Deserialize, Serialize, IntoParams, Debug)]
#[into_params(parameter_in=Query)]
pub struct QueryTimerange {
    /// Show results starting at this time. Can be YYYY, YYYY/MM, YYYY/MM/DD or YYYYwWW (for week)
    #[param(example = "2021/10")]
    from: Option<String>,
    /// Show results ending at this time. Can be YYYY, YYYY/MM, YYYY/MM/DD or YYYYwWW (for week)
    #[param(example = "2021/12")]
    to: Option<String>,
    /// Show results during this time range. Can be YYYY, YYYY/MM, YYYY/MM/DD or YYYYwWW (for week). Takes precedence over from and to
    #[param(example = "2024")]
    during: Option<String>,
}
impl MalojaQuery for QueryTimerange {}

impl QueryTimerange {
    pub fn to_timerange(&self) -> Result<ValidTimeRange, MalojaError> {
        match (&self.during, &self.from, &self.to) {
            (Some(during), None, None) => ValidTimeRange::from_string(during),
            (None, from, to) => ValidTimeRange::from_start_and_end_string(from.as_deref(), to.as_deref()),
            _ => Err(MalojaError::QueryError {
                message: "Can either specify a timerange with during, or start and end with from and to; not combine them".to_string(),
            })
        }
    }
}
impl ValidTimeRange {
    pub fn to_query(&self) -> QueryTimerange {
        let strings = self.to_strings();
        QueryTimerange {
            during: strings.0,
            from: strings.1,
            to: strings.2,
        }
    }
}

#[derive(Deserialize, Serialize, IntoParams, Debug)]
#[into_params(parameter_in=Query)]
pub struct QueryTimesteps {
    /// `day`, `week`, `month` or `year`
    #[param(example = "month")]
    pub step: String,
}
impl MalojaQuery for QueryTimesteps {}

const RANGE_IDENTIFIERS: [(&str, RangeType); 4] = [
    ("day", RangeType::Day),
    ("week", RangeType::Week),
    ("month", RangeType::Month),
    ("year", RangeType::Year),
];

impl QueryTimesteps {
    pub fn to_rangetype(&self) -> Result<RangeType, MalojaError> {
        let map: HashMap<&str, RangeType> = RANGE_IDENTIFIERS.iter().cloned().collect();
        let keys: Vec<&str> = map.keys().cloned().collect();
        map.get(self.step.as_str())
            .cloned()
            .ok_or(
                MalojaError::ParseError { message: format!("Valid range types are: {}", keys.join(", ")) }
            )
    }
}
impl RangeType {
    pub fn to_query(&self) -> QueryTimesteps {
        QueryTimesteps {
            step: match self {
                RangeType::Day => "day".to_string(),
                RangeType::Week => "week".to_string(),
                RangeType::Month => "month".to_string(),
                RangeType::Year => "year".to_string(),
            },
        }
    }
}

#[derive(Deserialize, Serialize, IntoParams, Debug)]
#[into_params(parameter_in=Query)]
pub struct QueryLimitArtist {
    /// Limit the output to this artist
    #[param(example = 69)]
    artist: Option<u32>,
}
impl MalojaQuery for QueryLimitArtist {}

impl QueryLimitArtist {
    pub fn to_artist_id(&self) -> Option<u32> {
        self.artist //do we really want an extra function here
    }
}
impl ArtistRead {
    pub fn to_query(&self) -> QueryLimitArtist {
        QueryLimitArtist { artist: Some(self.id) }
    }
}

#[derive(Deserialize, Serialize, IntoParams, Debug)]
#[into_params(parameter_in=Query)]
pub struct QueryLimitAlbum {
    /// Limit the output to this album
    #[param(example = 42)]
    album: Option<u32>,
}
impl MalojaQuery for QueryLimitAlbum {}

impl QueryLimitAlbum {
    pub fn to_album_id(&self) -> Option<u32> {
        self.album
    }
}

impl AlbumRead {
    pub fn to_query(&self) -> QueryLimitAlbum {
        QueryLimitAlbum { album: Some(self.id) }
    }
}

#[derive(Deserialize, Serialize, IntoParams, Debug)]
#[into_params(parameter_in=Query)]
pub struct QueryLimitTrack {
    /// Limit the output to this track
    #[param(example = 1337)]
    track: Option<u32>,
}
impl MalojaQuery for QueryLimitTrack {}

impl QueryLimitTrack {
    pub fn to_track_id(&self) -> Option<u32> {
        self.track
    }
}

impl TrackRead {
    pub fn to_query(&self) -> QueryLimitTrack {
        QueryLimitTrack { track: Some(self.id) }
    }
}

// i really don't like this, but this is basically
/// A common struct to return the artist / track / album to which a certain
/// page result was limited. It provides its own `to_querystring` since [`MalojaQuery`]
/// cannot be implemented on an Option.
pub struct EntityLimit {
    pub artist: Option<ArtistRead>,
    pub track: Option<TrackRead>,
    pub album: Option<AlbumRead>,
}
impl EntityLimit {
    pub fn to_querystring(&self) -> String {
        if let Some(artist) = &self.artist {
            format!("artist={}", artist.id)
        } else if let Some(track) = &self.track {
            format!("track={}", track.id)
        } else if let Some(album) = &self.album {
            format!("album={}", album.id)
        } else {
            "".to_string()
        }
    }
}

/// For when the entity isn't a query argument, but in the path (because we're on the main page of that entity)
#[derive(Deserialize, IntoParams, Debug)]
#[into_params(parameter_in=Path)]
pub struct PathEntity {
    pub id: u32,
}
