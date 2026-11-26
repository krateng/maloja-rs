// Internal helper structs for different stats, match the API schema closely

use crate::timeranges::{ValidTimeRange};
use serde::Serialize;


#[derive(Serialize, Clone, Debug)]
pub struct ChartsEntry<T> {
    pub rank: usize, //usize for the loop comparison in askama
    pub entry: T,
    pub scrobbles: u32,
    //pub seconds: u32
}
#[derive(Serialize, Clone, Debug)]
pub struct TopEntry<T> {
    pub time_range: ValidTimeRange,
    pub entry: T,
    pub scrobbles: u32,
    //pub seconds: u32
}

#[derive(Serialize, Clone, Debug)]
pub struct PulseEntry {
    pub time_range: ValidTimeRange,
    pub scrobbles: u32,
}

#[derive(Serialize, Clone, Debug)]
pub struct PerformanceEntry {
    pub time_range: ValidTimeRange,
    pub rank: u32,
}
