use axum::http::StatusCode;
use axum::Json;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use utoipa::{OpenApi, ToSchema};
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use crate::api::ScrobbleAPI;
use crate::database;
use crate::entity::artist::{Model as Artist, ArtistRead};
use crate::entity::track::{Model as Track, TrackRead};
use crate::entity::scrobble::{Model as Scrobble, ScrobbleRead};
use crate::entity::album::{Model as Album, AlbumRead};
use crate::database::repository::*;
use crate::database::views::{Charts, PaginationInfo, Top};


pub const API: ScrobbleAPI = ScrobbleAPI {
    prefix: "/maloja_2",
    tag: "Maloja v2",
    register: register_routes,
};

fn register_routes(mut router: OpenApiRouter) -> OpenApiRouter {
    router = router
        .routes(routes!(charts_tracks))
        .routes(routes!(charts_artists));
    router
}

#[derive(OpenApi)]
#[openapi(
    paths(charts_tracks, charts_artists),
    info(title = "Maloja API", version = "2"),
    components(schemas(ScrobbleRead,TrackRead,ArtistRead,AlbumRead))
)]
pub struct ApiDoc;


#[derive(Serialize, ToSchema)]
#[schema(title = "Error")]
struct APIError {
    description: String,
}
impl APIError {

}




#[utoipa::path(
    get,
    path = "/charts_tracks",
    responses(
        (status = OK, body = inline(Charts<TrackRead>)),
        (status = INTERNAL_SERVER_ERROR, body = inline(APIError))
    )
)]
pub async fn charts_tracks() -> Response {
    match database::repository::charts_tracks().await {
        Ok(tracks) => (StatusCode::OK, Json(Charts {
            pagination: PaginationInfo {
                page: 1,
                pages: 1,
                items_per_page: tracks.len() as u32,
                items_total: tracks.len() as u32,
            },
            result: tracks
        })).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(APIError { description: format!("{}", e) })).into_response(),
    }
}

#[utoipa::path(
    get,
    path = "/charts_artists",
    responses(
        (status = OK, body = inline(Charts<ArtistRead>)),
        (status = INTERNAL_SERVER_ERROR, body = inline(APIError))
    )
)]
pub async fn charts_artists() -> Response {
    match database::repository::charts_artists().await {
        Ok(artists) => (StatusCode::OK, Json(Charts {
            pagination: PaginationInfo {
                page: 1,
                pages: 1,
                items_per_page: artists.len() as u32,
                items_total: artists.len() as u32,
            },
            result: artists
        })).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(APIError { description: format!("{}", e) })).into_response(),
    }
}


