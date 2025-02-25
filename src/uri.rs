use std::cmp::min;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use utoipa::IntoParams;
use regex::Regex;
use crate::database::errors::MalojaError;
use crate::database::views::{Paginated, PaginationInfo};
use crate::timeranges::{TimeRange, BaseTimeRange, ALL_TIME, RangeType};

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
        let start_index = ((page-1) * per_page) as usize;
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
            result: results_slice
        }
    }
}

#[derive(Deserialize, IntoParams, Debug)]
#[into_params(parameter_in=Query)]
pub struct QueryTimerange {
    /// Show results starting at this time. Can be YYYY, YYYY/MM, YYYY/MM/DD or YYYYwWW (for week)
    #[param(example="2021/10")]
    from: Option<String>,
    /// Show results ending at this time. Can be YYYY, YYYY/MM, YYYY/MM/DD or YYYYwWW (for week)
    #[param(example="2021/12")]
    to: Option<String>,
    /// Show results during this time range. Can be YYYY, YYYY/MM, YYYY/MM/DD or YYYYwWW (for week). Takes precedence over from and to
    #[param(example="2024")]
    during: Option<String>,
}

impl QueryTimerange {
    pub fn to_timerange(&self) -> Result<TimeRange, MalojaError> {
        if self.during.is_some() && (self.from.is_some() || self.to.is_some()) {
            return Err(MalojaError::ParseError {
                message: "Can either specify a timerange with during, or start and end with from and to; not combine them".to_string(),
            })
        }
        if self.during.is_none() && self.from.is_none() && self.to.is_none() {
            return Ok(ALL_TIME);
        }
        if let Some(during) = &self.during {
            Ok(TimeRange::Simple(Self::match_string(during)?))
        }
        else {
            let tr = TimeRange::Composite {
                start: if let Some(from) = &self.from { Some(Self::match_string(from)?) } else { None },
                end: if let Some(to) = &self.to { Some(Self::match_string(to)?) } else { None },
            };
            if tr.validate() {
                Ok(tr)
            }
            else {
                Err(MalojaError::ParseError {
                    message: "From range must be before to range".to_string()
                })
            }
        }
    }

    fn match_string(input: &str) -> Result<BaseTimeRange, MalojaError> {
        if let Some(caps) = Regex::new(r"^(\d{3,4})$").unwrap().captures(input) {
            return Ok(BaseTimeRange::Year { year: caps[1].parse().unwrap() });
        }
        if let Some(caps) = Regex::new(r"^(\d{3,4})/(1[0-2]|0?[1-9])$").unwrap().captures(input) {
            return Ok(BaseTimeRange::Month { year: caps[1].parse().unwrap(), month: caps[2].parse().unwrap() });
        }
        if let Some(caps) = Regex::new(r"^(\d{3,4})/(1[0-2]|0?[1-9])/(3[01]|[12][0-9]|0?[1-9])$").unwrap().captures(input) {
            return Ok(BaseTimeRange::Day { year: caps[1].parse().unwrap(), month: caps[2].parse().unwrap(), day: caps[3].parse().unwrap() });
        }
        if let Some(caps) = Regex::new(r"^(\d{3,4})w(5[0-2]|[1-4][0-9]|[1-9])$").unwrap().captures(input) {
            return Ok(BaseTimeRange::Week { year: caps[1].parse().unwrap(), week: caps[2].parse().unwrap() });
        }
        Err(MalojaError::ParseError {
            message: "Timerange could not be parsed".to_string(),
        })

    }
}

#[derive(Deserialize, IntoParams, Debug)]
#[into_params(parameter_in=Query)]
pub struct QueryTimesteps {
    /// `day`, `week`, `month` or `year`
    #[param(example="month")]
    pub step: String
}
impl QueryTimesteps {
    pub fn to_type(&self) -> Result<RangeType, MalojaError> {
        match self.step.as_str() {
            "day" => Ok(RangeType::Day),
            "week" => Ok(RangeType::Week),
            "month" => Ok(RangeType::Month),
            "year" => Ok(RangeType::Year),
            _ => Err(MalojaError::ParseError { message: "Unknown range type".to_string() }),
        }
    }
}

#[derive(Deserialize, IntoParams, Debug)]
#[into_params(parameter_in=Query)]
pub struct QueryLimitArtist {
    /// Limit the output to this artist
    #[param(example=69)]
    artist: Option<u32>
}

impl QueryLimitArtist {
    pub fn to_artist_id(&self) -> Option<u32> {
        self.artist //do we really want an extra function here
    }
}

#[derive(Deserialize, IntoParams, Debug)]
#[into_params(parameter_in=Query)]
pub struct QueryLimitAlbum {
    /// Limit the output to this album
    #[param(example=42)]
    album: Option<u32>
}

impl QueryLimitAlbum {
    pub fn to_album_id(&self) -> Option<u32> {
        self.album
    }
}

#[derive(Deserialize, IntoParams, Debug)]
#[into_params(parameter_in=Query)]
pub struct QueryLimitTrack{
    /// Limit the output to this track
    #[param(example=1337)]
    track: Option<u32>
}

impl QueryLimitTrack {
    pub fn to_track_id(&self) -> Option<u32> {
        self.track
    }
}

#[derive(Deserialize, IntoParams, Debug)]
#[into_params(parameter_in=Path)]
pub struct PathEntity {
    pub id: u32
}