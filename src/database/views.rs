// These are essentially the API schemas, but we store them here because they will also be used internally

use std::time::Duration;
use serde::Serialize;
use utoipa::ToSchema;


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
    pub page: u32,
    pub pages: u32,
    pub items_per_page: u32,
    pub items_total: u32,
}
#[derive(Serialize, ToSchema, Clone, Debug)]
pub struct ChartsEntry<T> {
    pub rank: u32,
    pub entry: T,
    pub scrobbles: u32,
    //pub seconds: u32
}
#[derive(Serialize, ToSchema, Clone, Debug)]
pub struct TopEntry<T> {
    pub time_range: u32,
    pub entry: T,
    pub scrobbles: u32,
    //pub seconds: u32
}
