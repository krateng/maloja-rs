// These are essentially the API schemas, but we store them here because they will also be used internally

use std::time::Duration;
use serde::Serialize;
use utoipa::ToSchema;
use crate::timeranges::TimeRange;

#[derive(Serialize, ToSchema)]
pub struct Charts<T: Clone> {
    #[schema(inline)]
    pub pagination: PaginationInfo,
    #[schema(inline)]
    pub result: Vec<ChartsEntry<T>>
}
#[derive(Serialize, ToSchema)]
pub struct Top<T: Clone> {
    #[schema(inline)]
    pagination: PaginationInfo,
    #[schema(inline)]
    result: Vec<TopEntry<T>>
}

#[derive(Serialize, ToSchema)]
pub struct Paginated<T: Clone> {
    #[schema(inline)]
    pub(crate) pagination: PaginationInfo,
    #[schema(inline)]
    pub(crate) result: Vec<T>
}


#[derive(Serialize, ToSchema, Clone, Debug)]
pub struct PaginationInfo {
    #[schema(examples(2))]
    pub page: u32,
    #[schema(examples(5))]
    pub pages: u32,
    #[schema(examples(50))]
    pub items_per_page: u32,
    #[schema(examples(244))]
    pub items_total: u32,
}
#[derive(Serialize, ToSchema, Clone, Debug)]
pub struct ChartsEntry<T> {
    #[schema(examples(3))]
    pub rank: usize, //usize for the loop comparison in askama
    pub entry: T,
    #[schema(examples(1337))]
    pub scrobbles: u32,
    //pub seconds: u32
}
#[derive(Serialize, ToSchema, Clone, Debug)]
pub struct TopEntry<T> {
    #[schema(value_type = String, examples("2024"))]
    pub time_range: TimeRange,
    pub entry: T,
    #[schema(examples(313))]
    pub scrobbles: u32,
    //pub seconds: u32
}

#[derive(Serialize, ToSchema, Clone, Debug)]
pub struct PulseEntry {
    #[schema(value_type = String, examples("2024"))]
    pub time_range: TimeRange,
    #[schema(examples(313))]
    pub scrobbles: u32,
}

#[derive(Serialize, ToSchema, Clone, Debug)]
pub struct PerformanceEntry {
    #[schema(value_type = String, examples("2024"))]
    pub time_range: TimeRange,
    #[schema(examples(3))]
    pub rank: u32,
}