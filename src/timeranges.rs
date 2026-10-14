use chrono::{naive::Days, DateTime, Datelike, Local, Month, Months, NaiveDate, NaiveTime, NaiveWeek, TimeDelta, Utc, Weekday};
use chrono_tz::Tz;
use serde::{Serialize, Serializer};
use std::fmt::{Display, Formatter};
use std::iter::successors;
use std::ops::Add;
use sea_orm::prelude::Date;
use sea_query::Iden;
use sea_query::prelude::rust_decimal::prelude::FromPrimitive;
use strum_macros::Display;
use utoipa::ToSchema;
use crate::database::errors::MalojaError;
use crate::timeranges::RangeType::Day;

const TIMEZONE: Tz = Tz::Europe__Vienna; //AEIOU
const WEEK_BEGIN: Weekday = Weekday::Sun;
const FIRST_STAMP: i64 = 825092900; //TODO
const OUT_OF_RANGE_PANIC: &str = "How are you still using Maloja???";
const TIMEZONE_MIDNIGHT_PANIC: &str = "Please do not specify a timezone that can skip midnight.";

pub const ALL_TIME: ValidTimeRange = ValidTimeRange(TimeRange::Composite { start: None, end: None });

/// The basic time range types that correspond to gregorian units
#[derive(Clone, Debug, Eq, PartialEq)]
enum BaseTimeRange {
    Day { date: NaiveDate },
    Week { week: NaiveWeek },
    Month { year: u16, month: Month },
    Year { year: u16 },
}
impl BaseTimeRange {
    fn year(year: u16) -> Result<Self, MalojaError> {
        Ok(Self::Year { year })
    }
    fn month(year: u16, month: u8) -> Result<Self, MalojaError> {
        let month = Month::from_u8(month-1).ok_or(MalojaError::ParseError {
            message: format!("{} is not a valid month number", month),
        })?;
        Ok(Self::Month { year, month })
    }
    fn day(year: u16, month: u8, day: u8) -> Result<Self, MalojaError> {
        let date = NaiveDate::from_ymd_opt(year as i32, month as u32, day as u32).ok_or(MalojaError::ParseError {
            message: format!("{}/{}/{} is not a valid day", year, month, day),
        })?;
        Ok(Self::Day { date })
    }
    fn week(year: u16, week: u8) -> Result<Self, MalojaError> {
        let week_year_start = get_pseudo_week_year_start(year as i32);

        if week < 1 || week > 53 {
            // we accept W53 as W1 of the next year in years when it doesnt exist
            // if i understand my own code correctly, this will create an identical object
            // as if we had specified it correctly and display as such
            return Err(MalojaError::ParseError {
                message: "Week number must be from 1 to 53".to_string(),
            });
        }

        let week_start = week_year_start.checked_add_days(Days::new(((week as u64)-1) * 7)).expect(&OUT_OF_RANGE_PANIC);
        Ok(Self::Week { week: week_start.week(WEEK_BEGIN) })

    }
}

/// A fieldless enum simply to select the different types of time ranges as used by [`BaseTimeRange`]
#[derive(Clone, Debug, Display, Eq, PartialEq)]
pub enum RangeType {
    Day,
    Week,
    Month,
    Year,
}
impl RangeType {
    pub fn frequency_adv(&self) -> String {
        match self {
            RangeType::Day => String::from("Daily"),
            RangeType::Week => String::from("Weekly"),
            RangeType::Month => String::from("Monthly"),
            RangeType::Year => String::from("Yearly"),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
// TODO better toschema
pub enum TimeRange {
    Simple(BaseTimeRange),
    Composite { start: Option<BaseTimeRange>, end: Option<BaseTimeRange> },
}

// this should be the only one we construct from outside
// the inner one is still public to make pattern matching work,
// but you cant create it without access to basetimerange
pub struct ValidTimeRange(TimeRange);

impl ValidTimeRange {
    pub fn year(year: u16) -> Result<Self, MalojaError> {
        Ok(ValidTimeRange(TimeRange::Simple(BaseTimeRange::year(year)?)))
    }
    pub fn month(year: u16, month: u8) -> Result<Self, MalojaError> {
        Ok(ValidTimeRange(TimeRange::Simple(BaseTimeRange::month(year, month)?)))
    }
    pub fn day(year: u16, month: u8, day: u8) -> Result<Self, MalojaError> {
        Ok(ValidTimeRange(TimeRange::Simple(BaseTimeRange::day(year, month, day)?)))
    }
    pub fn week(year: u16, week: u8) -> Result<Self, MalojaError> {
        Ok(ValidTimeRange(TimeRange::Simple(BaseTimeRange::week(year, week)?)))
    }

    pub fn composite() -> Self {
        ValidTimeRange(TimeRange::Composite { start: None, end: None })
    }
    pub fn infinite() -> Self {
        Self::composite()
    }

    fn set_new_start(mut self, new: BaseTimeRange) -> Result<Self, MalojaError> {
        match &mut self.0 {
            TimeRange::Composite { start, end } => {
                if let Some(end) = end && new.naive_date_boundaries().0 > end.naive_date_boundaries().1 {
                    return Err(MalojaError::ParseError { message: "Start Range cannot be after End Range".to_string() });
                }
                *start = Some(new);
                Ok(self)
            }
            _ => {
                Err(MalojaError::TimeError { message: "Cannot call composite methods on non-composite time range!".to_string() })
            }
        }
    }
    fn set_new_end(mut self, new: BaseTimeRange) -> Result<Self, MalojaError> {
        match &mut self.0 {
            TimeRange::Composite { start, end } => {
                if let Some(start) = start && start.naive_date_boundaries().0 > new.naive_date_boundaries().1 {
                    return Err(MalojaError::ParseError { message: "Start Range cannot be after End Range".to_string() });
                }
                *end = Some(new);
                Ok(self)
            }
            _ => {
                Err(MalojaError::TimeError { message: "Cannot call composite methods on non-composite time range!".to_string() })
            }
        }
    }
    pub fn with_start_year(mut self, year: u16) -> Result<Self, MalojaError> {
        let new = BaseTimeRange::year(year)?;
        self.set_new_start(new)
    }
    pub fn with_start_month(mut self, year: u16, month: u8) -> Result<Self, MalojaError> {
        let new = BaseTimeRange::month(year, month)?;
        self.set_new_start(new)
    }
    pub fn with_start_day(mut self, year: u16, month: u8, day: u8) -> Result<Self, MalojaError> {
        let new = BaseTimeRange::day(year, month, day)?;
        self.set_new_start(new)
    }
    pub fn with_start_week(mut self, year: u16, week: u8) -> Result<Self, MalojaError> {
        let new = BaseTimeRange::week(year, week)?;
        self.set_new_start(new)
    }
    pub fn with_end_year(mut self, year: u16) -> Result<Self, MalojaError> {
        let new = BaseTimeRange::year(year)?;
        self.set_new_end(new)
    }
    pub fn with_end_month(mut self, year: u16, month: u8) -> Result<Self, MalojaError> {
        let new = BaseTimeRange::month(year, month)?;
        self.set_new_end(new)
    }
    pub fn with_end_day(mut self, year: u16, month: u8, day: u8) -> Result<Self, MalojaError> {
        let new = BaseTimeRange::day(year, month, day)?;
        self.set_new_end(new)
    }
    pub fn with_end_week(mut self, year: u16, week: u8) -> Result<Self, MalojaError> {
        let new = BaseTimeRange::week(year, week)?;
        self.set_new_end(new)
    }
}


impl ValidTimeRange {
    pub fn datetime_boundaries(&self) -> (DateTime<Tz>, DateTime<Tz>) {
        match &self.0 {
            TimeRange::Simple(base) => base.datetime_boundaries(),
            TimeRange::Composite { start, end } => {
                (
                    match start {
                        None => DateTime::from_timestamp(FIRST_STAMP, 0).expect("Hard-coded timestamp should not fail").with_timezone(&TIMEZONE),
                        Some(start) => start.datetime_boundaries().0,
                    },
                    match end {
                        None => Utc::now()
                            .with_timezone(&TIMEZONE)
                            .date_naive()
                            .succ_opt().expect(OUT_OF_RANGE_PANIC)
                            .and_time(NaiveTime::MIN)
                            .and_local_timezone(TIMEZONE)
                            .earliest()
                            .expect(TIMEZONE_MIDNIGHT_PANIC),
                        Some(end) => end.datetime_boundaries().1,
                    }
                )
            }
        }
    }

    pub fn timestamp_boundaries(&self) -> (i64, i64) {
        let (start, end) = self.datetime_boundaries();
        (start.timestamp(), end.timestamp()-1)
    }

    pub fn previous(&self) -> Option<Self> {
        match &self.0 {
            TimeRange::Simple(_) => None,
            TimeRange::Composite { start: Some(start), end: Some(end) } => {
                todo!()
            },
            TimeRange::Composite { start: Some(start), end: None } => {
                // open ended ranges have exactly one previous - the complement
                Some(ValidTimeRange(TimeRange::Composite {
                    start: None, end: start.previous()
                }))
            }
            TimeRange::Composite { start: None, end } => {
                // ranges without a start have no previous
                None
            }
        }
    }
    pub fn next(&self) -> Option<Self> {
        match &self.0 {
            TimeRange::Simple(_) => None,
            TimeRange::Composite { start: Some(start), end: Some(end) } => {
                todo!()
            },
            TimeRange::Composite { start: None, end: Some(end) } => {
                // open started ranges have exactly one next - the complement
                Some(ValidTimeRange(TimeRange::Composite {
                    start: end.next(), end: None
                }))
            },
            TimeRange::Composite { start, end: None } => {
                // ranges without an end have no next
                None
            }
        }
    }

    pub fn includes_timestamp(&self, timestamp: i64) -> bool {
        let (first, last) = self.timestamp_boundaries();
        first <= timestamp && timestamp <= last
    }

    pub fn describe(&self) -> String {
        match &self.0 {
            TimeRange::Simple(base) => base.describe(),
            TimeRange::Composite { start: Some(start), end: Some(end) } => format!("{} to {}", start.describe(), end.describe()),
            TimeRange::Composite { start: Some(start), end: None } => format!("From {}", start.describe()),
            TimeRange::Composite { start: None, end: Some(end) } => format!("Until {}", end.describe()),
            TimeRange::Composite { start: None, end: None } => "All Time".to_string()
        }
    }

    pub fn describe_short(&self) -> String {
        match &self.0 {
            TimeRange::Simple(base) => base.describe_short(),
            TimeRange::Composite { start: Some(start), end: Some(end) } => format!("{}-{}", start.describe(), end.describe()),
            TimeRange::Composite { start: Some(start), end: None } => format!("{}+", start.describe()),
            TimeRange::Composite { start: None, end: Some(end) } => format!("-{}", end.describe()),
            TimeRange::Composite { start: None, end: None } => "ALL".to_string()
        }
    }

    pub fn describe_relative(&self) -> String {
        let now = Local::now();
        if self.includes_timestamp(now.timestamp()) {
            match self.0 {
                TimeRange::Simple(BaseTimeRange::Day {..}) => "Today".to_string(),
                TimeRange::Simple(BaseTimeRange::Week {..}) => "This Week".to_string(),
                TimeRange::Simple(BaseTimeRange::Month {..}) => "This Month".to_string(),
                TimeRange::Simple(BaseTimeRange::Year {..}) => "This Year".to_string(),
                _ => self.describe()
            }
        }
        else if let Some(a) = self.previous() && a.includes_timestamp(now.timestamp()) {
            match self.0 {
                TimeRange::Simple(BaseTimeRange::Day {..}) => "Yesterday".to_string(),
                TimeRange::Simple(BaseTimeRange::Week {..}) => "Last Week".to_string(),
                TimeRange::Simple(BaseTimeRange::Month {..}) => "Last Month".to_string(),
                TimeRange::Simple(BaseTimeRange::Year {..}) => "Last Year".to_string(),
                _ => self.describe()
            }
        }
        else if let Some(a) = self.next() && a.includes_timestamp(now.timestamp()) {
            match self.0 {
                TimeRange::Simple(BaseTimeRange::Day {..}) => "Tomorrow".to_string(),
                TimeRange::Simple(BaseTimeRange::Week {..}) => "Next Week".to_string(),
                TimeRange::Simple(BaseTimeRange::Month {..}) => "Next Month".to_string(),
                TimeRange::Simple(BaseTimeRange::Year {..}) => "Next Year".to_string(),
                _ => self.describe()
            }
        }
        else {
            self.describe()
            // TODO
        }
    }

    /// Returns all subranges of the requested type so that the total range of subranges at least covers
    /// the full original range, with the least possible overhang.
    /// This means requesting bigger types of subranges (Week->Years) can return more than one subrange
    pub fn get_subranges(&self, subrange_type: RangeType) -> Vec<Self> {
        let (first, last) = self.datetime_boundaries();
        let first_base_obj = match subrange_type {
            RangeType::Day => BaseTimeRange::Day { date: first.date_naive() },
            RangeType::Week => BaseTimeRange::Week { week: first.date_naive().week(WEEK_BEGIN) },
            RangeType::Month => BaseTimeRange::Month { year: first.year() as u16, month: Month::from_u32(first.date_naive().month()).expect("What") },
            RangeType::Year => BaseTimeRange::Year { year: first.year() as u16 },
        };
        let first_range = ValidTimeRange(TimeRange::Simple(first_base_obj));

        let ranges: Vec<ValidTimeRange> = successors(Some(first_range), |prev| {
            (!prev.includes_timestamp(last.timestamp()))
                .then(|| prev.next()).unwrap_or(None)
        }).collect();

        ranges
    }
}

impl BaseTimeRange {
    // we assume the object has been created via the methods and is therefore valid, since it will not be
    // possible to create it from outside directly - but obviously this isnt a hard constraint on the type
    // when inside this module
    fn naive_date_boundaries(&self) -> (NaiveDate, NaiveDate) {
        match self {
            BaseTimeRange::Day { date } => {
                (
                    *date,
                    date.checked_add_days(Days::new(1)).expect(&OUT_OF_RANGE_PANIC)
                )
            }
            BaseTimeRange::Week { week } => {
                (
                    week.first_day(),
                    week.last_day()
                )
            }
            BaseTimeRange::Month { year, month } => {
                let first_day = Date::from_ymd_opt(*year as i32, month.number_from_month(), 1).expect(&OUT_OF_RANGE_PANIC);
                (
                    first_day,
                    first_day.checked_add_months(Months::new(1)).expect(&OUT_OF_RANGE_PANIC)
                )
            }
            BaseTimeRange::Year { year } => {
                (
                    NaiveDate::from_ymd_opt(*year as i32, 1, 1).expect(&OUT_OF_RANGE_PANIC),
                    NaiveDate::from_ymd_opt((*year as i32)+1, 1, 1).expect(&OUT_OF_RANGE_PANIC)
                )
            }
        }
    }

    fn datetime_boundaries(&self) -> (DateTime<Tz>, DateTime<Tz>) {
        let (date_begin, date_after) = self.naive_date_boundaries();
        (
            local_midnight(&date_begin),
            local_midnight(&date_after)
        )
    }

    fn previous(&self) -> Option<Self> {
        match self {
            BaseTimeRange::Day { date } => {
                let day_before = date.checked_sub_days(Days::new(1)).expect(&OUT_OF_RANGE_PANIC);
                Some(BaseTimeRange::Day { date: day_before })
            }
            BaseTimeRange::Month { year, month } => {
                let m = month.pred();
                let y = if m == Month::December { year - 1 } else { *year };
                Some(BaseTimeRange::Month { year: y, month: m })
            }
            BaseTimeRange::Week { week } => {
                let first_day = week.checked_first_day().expect(&OUT_OF_RANGE_PANIC);
                let first_day_last_week = first_day.checked_sub_days(Days::new(7)).expect(&OUT_OF_RANGE_PANIC);
                Some(BaseTimeRange::Week { week: first_day_last_week.week(WEEK_BEGIN) })
            }
            BaseTimeRange::Year { year } => {
                Some(BaseTimeRange::Year { year: *year-1 })
            }
        }
    }
    fn next(&self) -> Option<Self> {
        match self {
            BaseTimeRange::Day { date } => {
                let day_after = date.checked_add_days(Days::new(1)).expect(&OUT_OF_RANGE_PANIC);
                Some(BaseTimeRange::Day { date: day_after })
            }
            BaseTimeRange::Month { year, month } => {
                let m = month.succ();
                let y = if m == Month::January { year + 1 } else { *year };
                Some(BaseTimeRange::Month { year: y, month: m })
            }
            BaseTimeRange::Week { week } => {
                let first_day = week.checked_first_day().expect(&OUT_OF_RANGE_PANIC);
                let first_day_next_week = first_day.checked_add_days(Days::new(7)).expect(&OUT_OF_RANGE_PANIC);
                Some(BaseTimeRange::Week { week: first_day_next_week.week(WEEK_BEGIN) })
            }
            BaseTimeRange::Year { year } => {
                Some(BaseTimeRange::Year { year: *year+1 })
            }
        }
    }

    fn describe(&self) -> String {
        match self {
            BaseTimeRange::Day { date } => date.format("%d. %B %Y").to_string(),
            BaseTimeRange::Week { week } => {
                let (y, w) = get_year_and_number_of_week(week);
                format!("Week {} {}", w, y)
            }
            BaseTimeRange::Month { year, month } => format!("{} {}", month.name(), year),
            BaseTimeRange::Year { year } => format!("{}", year),
        }
    }
    fn describe_short(&self) -> String {
        match self {
            BaseTimeRange::Day { date } => date.format("%Y/%B/%d").to_string(),
            BaseTimeRange::Week { week } => {
                let (y, w) = get_year_and_number_of_week(week);
                format!("{}W{}", y, w)
            }
            BaseTimeRange::Month { year, month } => format!("{}/{}", year, month.number_from_month()),
            BaseTimeRange::Year { year } => format!("{}", year),
        }
    }

}




impl RangeType {

    pub fn get_current(&self) -> ValidTimeRange {
        let now = Utc::now().with_timezone(&TIMEZONE);
        match self {
            RangeType::Day => ValidTimeRange(TimeRange::Simple(BaseTimeRange::Day { date: now.date_naive() })),
            RangeType::Week => ValidTimeRange(TimeRange::Simple(BaseTimeRange::Week { week: now.date_naive().week(WEEK_BEGIN) })),
            RangeType::Month => ValidTimeRange(TimeRange::Simple(BaseTimeRange::Month { year: now.year() as u16, month: Month::from_u32(now.month()).expect("What") })),
            RangeType::Year => ValidTimeRange(TimeRange::Simple(BaseTimeRange::Year { year: now.year() as u16 })),
        }
    }
}


// This is used for the askama rendering
impl Display for ValidTimeRange {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.describe())
    }
}

// This is used in the API
impl Serialize for ValidTimeRange {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.describe_short().serialize(serializer)
    }
}

// helpers

fn local_midnight(day: &NaiveDate) -> DateTime<Tz> {
    day.and_time(NaiveTime::MIN).and_local_timezone(TIMEZONE).earliest().expect(&TIMEZONE_MIDNIGHT_PANIC)
}

fn get_year_and_number_of_week(week: &NaiveWeek) -> (i32, u8) {
    let our_year = week.first_day().year();
    let next_year = our_year + 1;
    // cant be previous year if our week begin is in this year
    if get_pseudo_week_year_start(next_year) == week.first_day() {
        (next_year, 1)
    }
    else {
        let this_year_start = get_pseudo_week_year_start(our_year);
        let days_diff = (week.first_day() - this_year_start).num_days();
        assert_eq!(days_diff % 7, 0);
        (our_year, ((days_diff / 7)+1) as u8)
    }


}

/// Get the first day of the 'year' of weeks that corresponds to this year, i.e. W1-W52/W53 of this year
fn get_pseudo_week_year_start(year: i32) -> NaiveDate {
    let first_day_this_year = NaiveDate::from_ymd_opt(year, 1, 1).expect("Every year must have 1/1");
    // how many days into the week are we at the start of the year
    let week_offset: u8 = first_day_this_year.weekday().days_since(WEEK_BEGIN) as u8;

    let first_week_start =
        if week_offset < 3 {
            // if more than half of the week is in this year, count this as the first week (so the year to week shift is -this)
            first_day_this_year.checked_sub_days(Days::new(week_offset as u64))
        }
        else {
            // otherwise, count the next as the first week, so the remainder days are the year to week shift
            first_day_this_year.checked_add_days(Days::new((7-week_offset) as u64))
        }.expect(&OUT_OF_RANGE_PANIC);

    first_week_start
}