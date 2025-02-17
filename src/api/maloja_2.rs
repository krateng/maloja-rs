use axum::extract::Query;
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
use crate::timeranges::{TimeRange, BaseTimeRange, ALL_TIME};
use crate::uri::{QueryLimitAlbum, QueryLimitArtist, QueryTimerange};

pub const API: ScrobbleAPI = ScrobbleAPI {
    prefix: "/maloja_2",
    tag: "Maloja v2",
    register: register_routes,
};

fn register_routes(mut router: OpenApiRouter) -> OpenApiRouter {
    router = router
        .routes(routes!(charts_tracks))
        .routes(routes!(charts_artists))
        .routes(routes!(charts_albums));
    router
}

#[derive(OpenApi)]
#[openapi(
    paths(charts_tracks, charts_artists, charts_albums),
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
    params(QueryTimerange, QueryLimitArtist, QueryLimitAlbum),
    responses(
        (status = OK, body = inline(Charts<TrackRead>)),
        (status = INTERNAL_SERVER_ERROR, body = inline(APIError))
    )
)]
pub async fn charts_tracks(Query(params_time): Query<QueryTimerange>, Query(params_limit_artist): Query<QueryLimitArtist>, Query(params_limit_album): Query<QueryLimitAlbum>) -> Response {
    let timerange = params_time.to_timerange();
    let artist_id = params_limit_artist.to_artist_id();
    let album_id = params_limit_album.to_album_id();
    match database::repository::charts_tracks(timerange, artist_id, album_id).await {
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
    params(QueryTimerange),
    responses(
        (status = OK, body = inline(Charts<ArtistRead>)),
        (status = INTERNAL_SERVER_ERROR, body = inline(APIError))
    )
)]
pub async fn charts_artists(Query(params_time): Query<QueryTimerange>) -> Response {
    let timerange = params_time.to_timerange();
    match database::repository::charts_artists(timerange).await {
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

#[utoipa::path(
    get,
    path = "/charts_albums",
    params(QueryTimerange, QueryLimitArtist),
    responses(
        (status = OK, body = inline(Charts<AlbumRead>)),
        (status = INTERNAL_SERVER_ERROR, body = inline(APIError))
    )
)]
pub async fn charts_albums(Query(params_time): Query<QueryTimerange>, Query(params_limit_artist): Query<QueryLimitArtist>) -> Response {
    let timerange = params_time.to_timerange();
    let artist_id = params_limit_artist.to_artist_id();
    match database::repository::charts_albums(timerange, artist_id).await {
        Ok(albums) => (StatusCode::OK, Json(Charts {
            pagination: PaginationInfo {
                page: 1,
                pages: 1,
                items_per_page: albums.len() as u32,
                items_total: albums.len() as u32,
            },
            result: albums
        })).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(APIError { description: format!("{}", e) })).into_response(),
    }
}


