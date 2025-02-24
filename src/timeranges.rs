use std::fmt::{Display, Formatter};
use chrono::{naive::Days, DateTime, Datelike, TimeZone, Weekday, NaiveDate, NaiveDateTime, Months, Utc};
use chrono_tz::Tz;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use utoipa::{PartialSchema, ToSchema};
use utoipa::openapi::{RefOr, Schema};

const TIMEZONE: Tz = Tz::Europe__Vienna; //AEIOU
const WEEK_BEGIN: Weekday = Weekday::Sun;
const FIRST_STAMP: i64 = 825092900; //TODO

pub const ALL_TIME: TimeRange = TimeRange::Infinite {};



/// The basic time range types that correspond to gregorian units
#[derive(Clone, Debug, Serialize, ToSchema)]
pub enum BaseTimeRange {
    Day { year: i32, month: u8, day: u8 },
    Week { year: i32, week: u8 },
    Month { year: i32, month: u8 },
    Year { year: i32 },
}
/// A fieldless enum simply to select the different types of time ranges as used by [`BaseTimeRange`]
#[derive(Clone, Debug)]
pub enum RangeType {
    Day, Week, Month, Year
}

#[derive(Clone, Debug)]
// TODO better toschema
pub enum TimeRange {
    Simple(BaseTimeRange),
    Composite { start: Option<BaseTimeRange>, end: Option<BaseTimeRange> },
    Infinite, //represented by Composite { None, None } as well, remove?
}




impl BaseTimeRange {
    fn datetime_boundaries(&self) -> (DateTime<Tz>, DateTime<Tz>) {
        match self {
            BaseTimeRange::Day { year, month, day } => {
                let thisday = NaiveDate::from_ymd_opt(*year, *month as u32, *day as u32).unwrap();
                (
                    thisday.and_hms_opt(0, 0, 0).unwrap().and_local_timezone(TIMEZONE).unwrap(),
                    thisday.and_hms_opt(23, 59, 59).unwrap().and_local_timezone(TIMEZONE).unwrap(),
                )
            }
            BaseTimeRange::Week { year, week } => {
                let first_day_this_year = NaiveDate::from_ymd_opt(*year as i32, 1, 1).unwrap();
                // weekoffset = days ahead of last week start
                // weekoffset2 = days until next week start
                let weekoffset: i32 = ((7 + first_day_this_year.weekday().num_days_from_sunday() - WEEK_BEGIN.num_days_from_sunday()) % 7) as i32;
                let weekoffset2: i32 = 7 - weekoffset;
                let use_offset = if weekoffset > weekoffset2 { weekoffset2 } else { -weekoffset };
                let first_week_start = if (use_offset >= 0) {
                    first_day_this_year.checked_add_days(Days::new(use_offset as u64)).unwrap()
                } else {
                    first_day_this_year.checked_sub_days(Days::new(-use_offset as u64)).unwrap()
                };
                let firstday = first_week_start.checked_add_days(Days::new((7 * week) as u64)).unwrap();
                let lastday = firstday.checked_add_days(Days::new(6)).unwrap();
                (
                    firstday.and_hms_opt(0, 0, 0).unwrap().and_local_timezone(TIMEZONE).unwrap(),
                    lastday.and_hms_opt(23, 59, 59).unwrap().and_local_timezone(TIMEZONE).unwrap(),
                )

            }
            BaseTimeRange::Month { year, month } => {
                let firstday = NaiveDate::from_ymd_opt(*year, *month as u32, 1).unwrap();
                let lastday = firstday.checked_add_months(Months::new(1)).unwrap().checked_sub_days(Days::new(1)).unwrap();
                (
                    firstday.and_hms_opt(0, 0, 0).unwrap().and_local_timezone(TIMEZONE).unwrap(),
                    lastday.and_hms_opt(23, 59, 59).unwrap().and_local_timezone(TIMEZONE).unwrap(),
                )
            }
            BaseTimeRange::Year { year } => {
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
            BaseTimeRange::Day { .. } => {
                let new = first.checked_sub_days(Days::new(1)).unwrap();
                BaseTimeRange::Day { year: new.year(), month: new.month() as u8, day: new.day() as u8 }
            }
            BaseTimeRange::Week { year, week } => {
                //let new = first.checked_sub_days(Days::new(7)).unwrap();
                let (mut year, mut week) = (year.clone(), week - 1);
                if week < 1 {
                    year -= 1;
                    week += 52;
                }
                BaseTimeRange::Week { year, week }
            }
            BaseTimeRange::Month { .. } => {
                let new = first.checked_sub_months(Months::new(1)).unwrap();
                BaseTimeRange::Month { year: new.year(), month: new.month() as u8 }
            }
            BaseTimeRange::Year { year } => {
                BaseTimeRange::Year { year: year - 1}
            }

        }
    }

    fn next(&self) -> Self {
        let (first,last) = self.datetime_boundaries();
        match self {
            BaseTimeRange::Day { .. } => {
                let new = first.checked_add_days(Days::new(1)).unwrap();
                BaseTimeRange::Day { year: new.year(), month: new.month() as u8, day: new.day() as u8 }
            }
            BaseTimeRange::Week { year, week } => {
                //let new = first.checked_sub_days(Days::new(7)).unwrap();
                let (mut year, mut week) = (year.clone(), week + 1);
                if week > 52 {
                    year += 1;
                    week -= 52;
                }
                BaseTimeRange::Week { year, week }
            }
            BaseTimeRange::Month { .. } => {
                let new = first.checked_add_months(Months::new(1)).unwrap();
                BaseTimeRange::Month { year: new.year(), month: new.month() as u8 }
            }
            BaseTimeRange::Year { year } => {
                BaseTimeRange::Year { year: year + 1}
            }
        }
    }

    fn describe(&self) -> String {
        match &self {
            BaseTimeRange::Day { year, month, day } => { format!("{} {} {}", day, month, year) }
            BaseTimeRange::Week { year, week } => { format!("W{} {}", week, year) }
            BaseTimeRange::Month { year, month } => { format!("{} {}", month, year) }
            BaseTimeRange::Year { year } => { format!("{}", year) }
        }
    }

    fn describe_simple(&self) -> String {
        match &self {
            BaseTimeRange::Day { year, month, day } => { format!("{}/{}/{}", year, month, day) }
            BaseTimeRange::Week { year, week } => { format!("{}w{}", year, week) }
            BaseTimeRange::Month { year, month } => { format!("{}/{}", year, month) }
            BaseTimeRange::Year { year } => { format!("{}", year) }
        }
    }
}

impl TimeRange {

    pub fn timestamp_boundaries(&self) -> (i64, i64) {
        let (a, b) = self.datetime_boundaries();
        (a.timestamp(), b.timestamp())
    }

    fn datetime_boundaries(&self) -> (DateTime<Tz>, DateTime<Tz>) {
        // let min: DateTime<Tz> = DateTime::from_timestamp(i32::MIN as i64, 0).unwrap().with_timezone(&TIMEZONE);
        // let max: DateTime<Tz> = DateTime::from_timestamp(i32::MAX as i64, 0).unwrap().with_timezone(&TIMEZONE);

        let min: DateTime<Tz> = DateTime::from_timestamp(FIRST_STAMP, 0).unwrap().with_timezone(&TIMEZONE);
        let max: DateTime<Tz> = Utc::now().with_timezone(&TIMEZONE).date_naive().and_hms_opt(0,0,0).unwrap().and_local_timezone(TIMEZONE).unwrap();

        match self {
            TimeRange::Simple(base) => {
                base.datetime_boundaries()
            }
            TimeRange::Composite { start, end } => {
                (
                    if let Some(start) = start { start.datetime_boundaries().0 } else { min },
                    if let Some(end) = end { end.datetime_boundaries().1 } else { max }
                )
            }
            TimeRange::Infinite => {
                (min,max)
            }
        }
    }

    fn previous(&self) -> Option<Self> {
        match self {
            TimeRange::Simple(base) => {
                Some(TimeRange::Simple(base.previous()))
            }
            TimeRange::Composite { start, end } => {
                todo!()
            }
            TimeRange::Infinite {} => {
                None
            }
        }
    }

    fn next(&self) -> Option<Self> {
        match self {
            TimeRange::Simple(base) => {
                Some(TimeRange::Simple(base.next()))
            }
            TimeRange::Composite { start, end } => {
                todo!()
            }
            TimeRange::Infinite {} => {
                None
            }
        }
    }

    pub fn includes(&self, timestamp: i64) -> bool {
        let (start, end) = self.timestamp_boundaries();
        (start <= timestamp) && (timestamp <= end)
    }
    
    pub fn validate(&self) -> bool {
        // TODO: i don't really like this being done here
        match self {
            TimeRange::Simple(base) => true,
            TimeRange::Composite { start, end } => {
                let (s,e) = &self.timestamp_boundaries();
                (s < e)
            }
            TimeRange::Infinite {} => true,
        }
    }

    pub fn get_subranges(&self, subrange_type: RangeType) -> Vec<TimeRange> {
        let (first, last) = self.datetime_boundaries();
        let mut result = vec![];
        let first_base_obj = match subrange_type {
            RangeType::Day => BaseTimeRange::Day { year: first.year(), month: first.month() as u8, day: first.day() as u8 },
            RangeType::Week => BaseTimeRange::Week { year: first.year(), week: first.iso_week().week() as u8 }, //TODO actual week logic
            RangeType::Month => BaseTimeRange::Month { year: first.year(), month: first.month() as u8 },
            RangeType::Year => BaseTimeRange::Year { year: first.year() },
        };
        let mut next = TimeRange::Simple(first_base_obj);
        loop {
            result.push(next.clone());
            if next.includes(last.timestamp()) {
                break;
            }
            next = next.next().unwrap();
        }
        result
    }

    fn describe(&self) -> String {
        match &self {
            TimeRange::Simple(base) => { format!("{}", base) }
            TimeRange::Composite { start, end } => {
                match (start, end) {
                    (Some(start), Some(end)) => { format!("{} to {}", start, end) }
                    (Some(start), None) => { format!("From {}", start) }
                    (None, Some(end)) => { format!("Until {}", end) }
                    (None, None) => { format!("{}", "All Time") }
                }

            }
            TimeRange::Infinite => { format!("{}", "All Time") }
        }
    }
    fn describe_simple(&self) -> String {
        match &self {
            TimeRange::Simple(base) => { format!("{}", base.describe_simple()) }
            TimeRange::Composite { start, end } => {
                match (start, end) {
                    (Some(start), Some(end)) => { format!("{} - {}", start.describe_simple(), end.describe_simple()) }
                    (Some(start), None) => { format!("{} -", start.describe_simple()) }
                    (None, Some(end)) => { format!("- {}", end.describe_simple()) }
                    (None, None) => { format!("{}", "ALL") }
                }

            }
            TimeRange::Infinite => { format!("{}", "All Time") }
        }
    }
}

// This is used for the askama rendering
impl Display for BaseTimeRange {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.describe())

    }
}
impl Display for TimeRange {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.describe())
    }
}

// This is used in the API
impl Serialize for TimeRange {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer
    {
        self.describe_simple().serialize(serializer)
    }
}


