use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use utoipa::IntoParams;
use regex::Regex;
use crate::timeranges::{TimeRange, BaseTimeRange, ALL_TIME};

// Query args
pub struct QueryPagination {
    page: Option<u32>,
    per_page: Option<u32>,
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
    pub fn to_timerange(&self) -> TimeRange {
        if let Some(during) = &self.during {
            let timerange = Self::match_string(during);
            timerange
        }
        else {
            ALL_TIME
        }
    }

    fn match_string(input: &str) -> TimeRange {
        if let Some(caps) = Regex::new(r"^(\d{3,4})$").unwrap().captures(input) {
            return TimeRange::Simple { unit: BaseTimeRange::Year { year: caps[1].parse().unwrap() } };
        }
        if let Some(caps) = Regex::new(r"^(\d{3,4})/(1[0-2]|0?[1-9])$").unwrap().captures(input) {
            return TimeRange::Simple { unit: BaseTimeRange::Month { year: caps[1].parse().unwrap(), month: caps[2].parse().unwrap() } };
        }
        if let Some(caps) = Regex::new(r"^(\d{3,4})/(1[0-2]|0?[1-9])/(3[01]|[12][0-9]|0?[1-9])$").unwrap().captures(input) {
            return TimeRange::Simple { unit: BaseTimeRange::Day { year: caps[1].parse().unwrap(), month: caps[2].parse().unwrap(), day: caps[3].parse().unwrap() } };
        }
        if let Some(caps) = Regex::new(r"^(\d{3,4})w(5[0-2]|[1-4][0-9]|[1-9])$").unwrap().captures(input) {
            return TimeRange::Simple { unit: BaseTimeRange::Week { year: caps[1].parse().unwrap(), week: caps[2].parse().unwrap() } };
        }
        ALL_TIME

    }
}

#[derive(Deserialize, IntoParams, Debug)]
#[into_params(parameter_in=Query)]
pub struct QueryLimitArtist {
    /// Limit the output to this artist
    #[param(example=69)]
    artist: Option<u32>
}

#[derive(Deserialize, IntoParams, Debug)]
#[into_params(parameter_in=Query)]
pub struct QueryLimitAlbum {
    /// Limit the output to this album
    #[param(example=42)]
    album: Option<u32>
}
