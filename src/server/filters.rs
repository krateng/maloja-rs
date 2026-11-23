use askama::Result as AskamaResult;
use chrono::{Duration, Utc};
use sea_query::Iden;
use time::macros::timestamp;

#[askama::filter_fn]
pub fn format_ts(timestamp: &i64, env: &dyn askama::Values) -> AskamaResult<String> {
    // TODO later use config obv
    let tz = chrono_tz::Tz::Europe__Vienna;
    let time = chrono::DateTime::from_timestamp(*timestamp, 0).unwrap();
    let fmt = "%Y-%m-%dT%H:%M:%S";
    let local_time = time.with_timezone(&tz);
    Ok(local_time.format(fmt).to_string())
}

#[askama::filter_fn]
pub fn format_ts_short(timestamp: &i64, env: &dyn askama::Values) -> AskamaResult<String> {
    Ok(format_short(*timestamp))
}

#[askama::filter_fn]
pub fn format_ts_short_allow_none(timestamp: &Option<i64>, env: &dyn askama::Values) -> AskamaResult<String> {
    match timestamp {
        Some(timestamp) => Ok(format_short(*timestamp)),
        None => Ok("Never".to_string()),
    }
}

fn format_short(timestamp: i64) -> String {
    let tz = chrono_tz::Tz::Europe__Vienna;
    let time = chrono::DateTime::from_timestamp(timestamp, 0).unwrap();
    let now = Utc::now();
    let diff = (now - time);
    if diff.num_seconds() < 120 {
        format!("{} seconds ago", diff.num_seconds())
    }
    else if diff.num_minutes() < 120 {
        format!("{} minutes ago", diff.num_minutes())
    }
    else if now.date_naive() == time.date_naive() {
        "Today".to_string()
    }
    else if now.date_naive()-Duration::days(1) == time.date_naive() {
        "Yesterday".to_string()
    }
    else {
        let fmt = "%Y-%m-%d";
        let local_time = time.with_timezone(&tz);
        local_time.format(fmt).to_string()
    }
}