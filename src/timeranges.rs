use std::ops::Add;
use chrono;
use chrono::{naive::Days, DateTime, Datelike, TimeZone, Weekday, NaiveDate, NaiveDateTime, Months};
use chrono_tz::Tz;


const TIMEZONE: Tz = Tz::Europe__Vienna; //AEIOU
const WEEK_BEGIN: Weekday = Weekday::Sun;


// Each time range needs to offer previous / next, its first and last timestamp
pub trait TimeRangeTrait {

    fn datetime_boundaries(&self) -> (DateTime<Tz>, DateTime<Tz>);
    fn timestamp_boundaries(&self) -> (i64, i64) {
        (self.datetime_boundaries().0.timestamp(), self.datetime_boundaries().1.timestamp())
    }
    fn previous(&self) -> Self;
    fn next(&self) -> Self;
    fn description(&self) -> String;
    fn description_with_preposition(&self) -> String;
}

pub trait SimpleTimeRangeTrait: TimeRangeTrait {}


pub enum SimpleTimeRange {
    Day { year: i32, month: u8, day: u8 },
    Week { year: i32, week: u8 },
    Month { year: i32, month: u8 },
    Year { year: i32 },
}
impl SimpleTimeRangeTrait for SimpleTimeRange {}

pub struct CustomTimeRange<T: SimpleTimeRangeTrait> {
    start: T,
    end: T,
}

impl TimeRangeTrait for SimpleTimeRange {
    fn datetime_boundaries(&self) -> (DateTime<Tz>, DateTime<Tz>) {
        match self {
            SimpleTimeRange::Day { year, month, day } => {
                let thisday = NaiveDate::from_ymd_opt(*year, *month as u32, *day as u32).unwrap();
                (
                    thisday.and_hms_opt(0, 0, 0).unwrap().and_local_timezone(TIMEZONE).unwrap(),
                    thisday.and_hms_opt(23, 59, 59).unwrap().and_local_timezone(TIMEZONE).unwrap(),
                )
            }
            SimpleTimeRange::Week { year, week } => {
                let first_day_this_year = NaiveDate::from_ymd_opt(*year as i32, 1, 1).unwrap();
                // weekoffset = days ahead of last week start
                // weekoffset2 = days until next week start
                let weekoffset: i32 = ((7 + first_day_this_year.weekday().num_days_from_sunday() - WEEK_BEGIN.num_days_from_sunday()) % 7) as i32;
                let weekoffset2: i32 = 7 - weekoffset;
                let use_offset = if weekoffset > weekoffset2 { weekoffset2 } else { -weekoffset };
                let first_week_start = first_day_this_year.checked_add_days(Days::new(use_offset as u64)).unwrap();
                let firstday = first_week_start.checked_add_days(Days::new((7 * week) as u64)).unwrap();
                let lastday = firstday.checked_add_days(Days::new(6)).unwrap();
                (
                    firstday.and_hms_opt(0, 0, 0).unwrap().and_local_timezone(TIMEZONE).unwrap(),
                    lastday.and_hms_opt(23, 59, 59).unwrap().and_local_timezone(TIMEZONE).unwrap(),
                )

            }
            SimpleTimeRange::Month { year, month } => {
                let firstday = NaiveDate::from_ymd_opt(*year, *month as u32, 1).unwrap();
                let lastday = firstday.checked_add_months(Months::new(1)).unwrap().checked_sub_days(Days::new(1)).unwrap();
                (
                    firstday.and_hms_opt(0, 0, 0).unwrap().and_local_timezone(TIMEZONE).unwrap(),
                    lastday.and_hms_opt(23, 59, 59).unwrap().and_local_timezone(TIMEZONE).unwrap(),
                )
            }
            SimpleTimeRange::Year { year } => {
                let firstday = NaiveDate::from_ymd_opt(*year as i32, 1, 1).unwrap();
                let lastday = NaiveDate::from_ymd_opt(*year as i32, 12, 31).unwrap();
                (
                    firstday.and_hms_opt(0, 0, 0).unwrap().and_local_timezone(TIMEZONE).unwrap(),
                    lastday.and_hms_opt(23, 59, 59).unwrap().and_local_timezone(TIMEZONE).unwrap(),
                )
            }
        }
    }

    fn previous(&self) -> Self {
        let (first,last) = self.datetime_boundaries();
        match self {
            SimpleTimeRange::Day { .. } => {
                let new = first.checked_sub_days(Days::new(1)).unwrap();
                SimpleTimeRange::Day { year: new.year(), month: new.month() as u8, day: new.day() as u8 }
            }
            SimpleTimeRange::Week { .. } => {
                let new = first.checked_sub_days(Days::new(7)).unwrap();
                SimpleTimeRange::Day { year: new.year(), month: new.month() as u8, day: new.day() as u8 }
            }
            SimpleTimeRange::Month { .. } => {
                let new = first.checked_sub_months(Months::new(1)).unwrap();
                SimpleTimeRange::Month { year: new.year(), month: new.month() as u8 }
            }
            SimpleTimeRange::Year { year } => {
                SimpleTimeRange::Year { year: year - 1}
            }
        }
    }

    fn next(&self) -> Self {
        let (first,last) = self.datetime_boundaries();
        match self {
            SimpleTimeRange::Day { .. } => {
                let new = first.checked_add_days(Days::new(1)).unwrap();
                SimpleTimeRange::Day { year: new.year(), month: new.month() as u8, day: new.day() as u8 }
            }
            SimpleTimeRange::Week { .. } => {
                let new = first.checked_add_days(Days::new(7)).unwrap();
                SimpleTimeRange::Day { year: new.year(), month: new.month() as u8, day: new.day() as u8 }
            }
            SimpleTimeRange::Month { .. } => {
                let new = first.checked_add_months(Months::new(1)).unwrap();
                SimpleTimeRange::Month { year: new.year(), month: new.month() as u8 }
            }
            SimpleTimeRange::Year { year } => {
                SimpleTimeRange::Year { year: year + 1}
            }
        }
    }

    fn description(&self) -> String {
        let (first,last) = self.datetime_boundaries();
        match self {
            SimpleTimeRange::Day { .. } => first.format("%d %m %Y").to_string(),
            SimpleTimeRange::Week { year, week } => format!("W{} {}", week, year),
            SimpleTimeRange::Month { .. } => first.format("%m %Y").to_string(),
            SimpleTimeRange::Year { .. } => first.format("%Y").to_string(),
        }
    }

    fn description_with_preposition(&self) -> String {
        let pre = match self {
            SimpleTimeRange::Day { .. } => String::from("on"),
            SimpleTimeRange::Week { .. } => String::from("in"),
            SimpleTimeRange::Month { .. } => String::from("in"),
            SimpleTimeRange::Year { .. } => String::from("in"),
        };
        format!("{} {}", pre, self.description())
    }
}

impl<T: SimpleTimeRangeTrait> TimeRangeTrait for CustomTimeRange<T> {
    fn datetime_boundaries(&self) -> (DateTime<Tz>, DateTime<Tz>) {
        (
            self.start.datetime_boundaries().0,
            self.end.datetime_boundaries().1,
        )
    }

    fn previous(&self) -> Self {
        todo!()
    }

    fn next(&self) -> Self {
        todo!()
    }

    fn description(&self) -> String {
        format!("{} through {}", self.start.description(), self.end.description())
    }

    fn description_with_preposition(&self) -> String {
        format!("from {} through {}", self.start.description(), self.end.description())
    }
}