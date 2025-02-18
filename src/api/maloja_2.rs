use axum::extract::{Path, Query};
use axum::http::StatusCode;
use axum::Json;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use utoipa::{OpenApi, ToSchema};
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use crate::api::ScrobbleAPI;
use crate::database;
use crate::database::errors::MalojaError;
use crate::entity::artist::{Model as Artist, ArtistRead};
use crate::entity::track::{Model as Track, TrackRead};
use crate::entity::scrobble::{Model as Scrobble, ScrobbleRead};
use crate::entity::album::{Model as Album, AlbumRead};
use crate::database::views::{Charts, PaginationInfo, Top};
use crate::timeranges::{TimeRange, BaseTimeRange, ALL_TIME};
use crate::uri::{PathEntity, QueryLimitAlbum, QueryLimitArtist, QueryTimerange};

pub const API: ScrobbleAPI = ScrobbleAPI {
    prefix: "/maloja_2",
    tag: "Maloja v2",
    register: register_routes,
};

fn register_routes(mut router: OpenApiRouter) -> OpenApiRouter {
    router = router
        .routes(routes!(info_artist))
        .routes(routes!(info_track))
        .routes(routes!(info_album))
        .routes(routes!(charts_tracks))
        .routes(routes!(charts_artists))
        .routes(routes!(charts_albums));
    router
}

#[derive(OpenApi)]
#[openapi(
    paths(charts_tracks, charts_artists, charts_albums, info_artist, info_album, info_track),
    info(title = "Maloja API", version = "2"),
    components(schemas(ScrobbleRead,TrackRead,ArtistRead,AlbumRead))
)]
pub struct ApiDoc;


#[derive(Serialize, ToSchema)]
#[schema(title = "Error")]
struct APIError {
    error: String,
}
impl APIError {
    fn new_response(code: StatusCode, message: String) -> Response {
        (code, Json(Self {
            error: message,
        })).into_response()
    }
}

impl IntoResponse for MalojaError {
    fn into_response(self) -> axum::response::Response {
        match &self {
            MalojaError::ArtistNotFound { id } => (StatusCode::NOT_FOUND, Json(APIError {
                error: format!("Artist {} not found", id),
            })).into_response(),
            MalojaError::TrackNotFound { id } => (StatusCode::NOT_FOUND, Json(APIError {
                error: format!("Track {} not found", id),
            })).into_response(),
            MalojaError::AlbumNotFound { id } => (StatusCode::NOT_FOUND, Json(APIError {
                error: format!("Album {} not found", id),
            })).into_response(),
            MalojaError::DatabaseConnectionError { message } => (StatusCode::INTERNAL_SERVER_ERROR, Json(message)).into_response(),
            e => (StatusCode::INTERNAL_SERVER_ERROR, Json(())).into_response(),
        }
    }
}


#[utoipa::path(
    get,
    path = "/artist/{id}",
    params(PathEntity),
    responses(
        (status = OK, body = ArtistRead),
        (status = INTERNAL_SERVER_ERROR, body = inline(APIError))
    )
)]
pub async fn info_artist(Path(params_path): Path<PathEntity>) -> Result<(StatusCode, Json<ArtistRead>), MalojaError> {
    let result = database::repository::artist_info(params_path.id).await?;
    Ok((StatusCode::OK, Json(result)))
}

#[utoipa::path(
    get,
    path = "/track/{id}",
    params(PathEntity),
    responses(
        (status = OK, body = TrackRead),
        (status = INTERNAL_SERVER_ERROR, body = inline(APIError))
    )
)]
pub async fn info_track(Path(params_path): Path<PathEntity>) -> Result<(StatusCode, Json<TrackRead>), MalojaError> {
    let result = database::repository::track_info(params_path.id).await?;
    Ok((StatusCode::OK, Json(result)))
}

#[utoipa::path(
    get,
    path = "/album/{id}",
    params(PathEntity),
    responses(
        (status = OK, body = AlbumRead),
        (status = INTERNAL_SERVER_ERROR, body = inline(APIError))
    )
)]
pub async fn info_album(Path(params_path): Path<PathEntity>) -> Result<(StatusCode, Json<AlbumRead>), MalojaError> {
    let result = database::repository::album_info(params_path.id).await?;
    Ok((StatusCode::OK, Json(result)))
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
pub async fn charts_tracks(
    Query(params_time): Query<QueryTimerange>,
    Query(params_limit_artist): Query<QueryLimitArtist>,
    Query(params_limit_album): Query<QueryLimitAlbum>
) -> Result<(StatusCode, Json<Charts<TrackRead>>), MalojaError> {
    let timerange = params_time.to_timerange();
    let artist_id = params_limit_artist.to_artist_id();
    let album_id = params_limit_album.to_album_id();
    let tracks = database::repository::charts_tracks(timerange, artist_id, album_id).await?;
    Ok((StatusCode::OK, Json(Charts {
        pagination: PaginationInfo {
            page: 1,
            pages: 1,
            items_per_page: tracks.len() as u32,
            items_total: tracks.len() as u32,
        },
        result: tracks
    })))
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
pub async fn charts_artists(
    Query(params_time): Query<QueryTimerange>
) -> Result<(StatusCode, Json<Charts<ArtistRead>>), MalojaError> {
    let timerange = params_time.to_timerange();
    let artists = database::repository::charts_artists(timerange).await?;
    Ok((StatusCode::OK, Json(Charts {
        pagination: PaginationInfo {
            page: 1,
            pages: 1,
            items_per_page: artists.len() as u32,
            items_total: artists.len() as u32,
        },
        result: artists
    })))
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
pub async fn charts_albums(
    Query(params_time): Query<QueryTimerange>,
    Query(params_limit_artist): Query<QueryLimitArtist>
) -> Result<(StatusCode, Json<Charts<AlbumRead>>), MalojaError> {
    let timerange = params_time.to_timerange();
    let artist_id = params_limit_artist.to_artist_id();
    let albums = database::repository::charts_albums(timerange, artist_id).await?;
    Ok((StatusCode::OK, Json(Charts {
        pagination: PaginationInfo {
            page: 1,
            pages: 1,
            items_per_page: albums.len() as u32,
            items_total: albums.len() as u32,
        },
        result: albums
    })))
}


