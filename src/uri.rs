use crate::database::errors::MalojaError;
use crate::database::views::{Paginated, PaginationInfo};
use crate::entity::album::AlbumRead;
use crate::entity::artist::ArtistRead;
use crate::entity::track::TrackRead;
use crate::timeranges::{BaseTimeRange, RangeType, TimeRange, ALL_TIME};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::cmp::min;
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
        let end_index = (start_index + per_page as usize);
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
    pub fn to_timerange(&self) -> Result<TimeRange, MalojaError> {
        if self.during.is_some() && (self.from.is_some() || self.to.is_some()) {
            return Err(MalojaError::ParseError {
                message: "Can either specify a timerange with during, or start and end with from and to; not combine them".to_string(),
            });
        }
        if self.during.is_none() && self.from.is_none() && self.to.is_none() {
            return Ok(ALL_TIME);
        }
        if let Some(during) = &self.during {
            Ok(TimeRange::Simple(Self::match_string(during)?))
        } else {
            let tr = TimeRange::Composite {
                start: if let Some(from) = &self.from {
                    Some(Self::match_string(from)?)
                } else {
                    None
                },
                end: if let Some(to) = &self.to { Some(Self::match_string(to)?) } else { None },
            };
            if tr.validate() {
                Ok(tr)
            } else {
                Err(MalojaError::ParseError {
                    message: "From range must be before to range".to_string(),
                })
            }
        }
    }

    fn match_string(input: &str) -> Result<BaseTimeRange, MalojaError> {
        if let Some(caps) = Regex::new(r"^(\d{3,4})$").unwrap().captures(input) {
            return Ok(BaseTimeRange::Year { year: caps[1].parse().unwrap() });
        }
        if let Some(caps) = Regex::new(r"^(\d{3,4})/(1[0-2]|0?[1-9])$").unwrap().captures(input) {
            return Ok(BaseTimeRange::Month {
                year: caps[1].parse().unwrap(),
                month: caps[2].parse().unwrap(),
            });
        }
        if let Some(caps) = Regex::new(r"^(\d{3,4})/(1[0-2]|0?[1-9])/(3[01]|[12][0-9]|0?[1-9])$")
            .unwrap()
            .captures(input)
        {
            return Ok(BaseTimeRange::Day {
                year: caps[1].parse().unwrap(),
                month: caps[2].parse().unwrap(),
                day: caps[3].parse().unwrap(),
            });
        }
        if let Some(caps) =
            Regex::new(r"^(\d{3,4})w(5[0-2]|[1-4][0-9]|[1-9])$").unwrap().captures(input)
        {
            return Ok(BaseTimeRange::Week {
                year: caps[1].parse().unwrap(),
                week: caps[2].parse().unwrap(),
            });
        }
        Err(MalojaError::ParseError { message: "Timerange could not be parsed".to_string() })
    }
}
impl TimeRange {
    pub fn to_query(&self) -> QueryTimerange {
        match self {
            TimeRange::Simple(base) => {
                QueryTimerange { from: None, to: None, during: Some(Self::base_to_str(base)) }
            }
            TimeRange::Composite { start, end } => QueryTimerange {
                from: start.clone().map(|x| Self::base_to_str(&x)),
                to: end.clone().map(|x| Self::base_to_str(&x)),
                during: None,
            },
            TimeRange::Infinite => QueryTimerange { from: None, to: None, during: None },
        }
    }

    fn base_to_str(input: &BaseTimeRange) -> String {
        match input {
            BaseTimeRange::Day { year, month, day } => format!("{}/{}/{}", year, month, day),
            BaseTimeRange::Week { year, week } => format!("{}w{}", year, week),
            BaseTimeRange::Month { year, month } => format!("{}/{}", year, month),
            BaseTimeRange::Year { year } => format!("{}", year),
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

impl QueryTimesteps {
    pub fn to_rangetype(&self) -> Result<RangeType, MalojaError> {
        match self.step.as_str() {
            "day" => Ok(RangeType::Day),
            "week" => Ok(RangeType::Week),
            "month" => Ok(RangeType::Month),
            "year" => Ok(RangeType::Year),
            _ => Err(MalojaError::ParseError { message: "Unknown range type".to_string() }),
        }
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
