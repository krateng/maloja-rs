use std::future::Future;
use axum::extract::{FromRequestParts, Query};
use axum::extract::path::ErrorKind;
use axum::extract::rejection::{JsonRejection, PathRejection, QueryRejection};
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum::Json;
use axum::response::{IntoResponse, Response};
use axum::routing::any;
use serde::de::DeserializeOwned;
use serde::Serialize;
use utoipa::{OpenApi, ToSchema};
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use strum_macros::{EnumString, Display};

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
        .routes(routes!(charts_albums))
        //.fallback(notfound); // TODO: https://github.com/tokio-rs/axum/issues/3138
        .route("/{*rest}", any(notfound));
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
    #[schema(example = "BadSong")]
    error_type: String,
    #[schema(example = "The submitted song is not a bop. Try listening to some (G)I-DLE instead!")]
    description: String,
}

fn create_response(e: &MalojaError, code: StatusCode, description: String) -> Response {
    (
        code,
        Json(APIError {
            error_type: format!("{}", e),
            //response_code: code.as_u16(),
            description
        })
    ).into_response()
}

impl IntoResponse for MalojaError {
    fn into_response(self) -> Response {
        match &self {
            MalojaError::ArtistNotFound { id } => create_response(&self, StatusCode::NOT_FOUND, format!("Artist {} not found", id)),
            MalojaError::TrackNotFound { id } => create_response(&self, StatusCode::NOT_FOUND, format!("Track {} not found", id)),
            MalojaError::AlbumNotFound { id } => create_response(&self, StatusCode::NOT_FOUND, format!("Album {} not found", id)),
            MalojaError::DatabaseConnectionError { message } => create_response(&self, StatusCode::INTERNAL_SERVER_ERROR, message.clone()),
            e => create_response(&self, StatusCode::INTERNAL_SERVER_ERROR, String::from("Unspecified Server Error")),
        }
    }
}


// custom path extractor to return proper json errors
struct Path<T>(T);

impl<S, T> FromRequestParts<S> for Path<T>
where
// these trait bounds are copied from `impl FromRequest for axum::extract::path::Path`
    T: DeserializeOwned + Send,
    S: Send + Sync,
{
    type Rejection = (StatusCode, Json<APIError>);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        match axum::extract::Path::<T>::from_request_parts(parts, state).await {
            Ok(value) => Ok(Self(value.0)),

            Err(rejection) => {
                let (status, body) = match rejection {
                    PathRejection::FailedToDeserializePathParams(inner) => {
                        let status = StatusCode::BAD_REQUEST;
                        let kind = inner.kind();
                        let description = match kind {
                            ErrorKind::ParseErrorAtKey { key, .. } => format!("Key <{}> could not be parsed", key),
                            _ => "Parameters could not be parsed".to_string(),
                        };
                        (status, APIError {
                            error_type: "RequestParamsParseError".to_string(),
                            description: description.to_string(),
                        })
                    }
                    PathRejection::MissingPathParams(error) => {
                        (StatusCode::BAD_REQUEST, APIError {
                            error_type: "RequestParamsParseError".to_string(),
                            description: "Missing path parameters".to_string(),
                        })
                    }
                    _ => {
                        (StatusCode::INTERNAL_SERVER_ERROR, APIError {
                            error_type: "RequestParamsParseError".to_string(),
                            description: "Unknown error while parsing path parameters".to_string(),
                        })
                    }
                };

                Err((status, Json(body)))
            }
        }
    }
}

pub async fn notfound() -> Response {
    (StatusCode::NOT_FOUND, Json(APIError {
        error_type: "InvalidEndpoint".to_string(),
        description: "Endpoint does not exist in this API.".to_string(),
    })).into_response()
}


#[utoipa::path(
    get,
    path = "/artist/{id}",
    params(PathEntity),
    responses(
        (status = OK, body = ArtistRead, description = "Successful request"),
        (status = NOT_FOUND, body = inline(APIError), description = "Artist ID does not exist in database"),
        (status = INTERNAL_SERVER_ERROR, body = inline(APIError), description = "Server error while handling the request"),
    )
)]
async fn info_artist(Path(params_path): Path<PathEntity>) -> Result<(StatusCode, Json<ArtistRead>), MalojaError> {
    let result = database::repository::artist_info(params_path.id).await?;
    Ok((StatusCode::OK, Json(result)))

}

#[utoipa::path(
    get,
    path = "/track/{id}",
    params(PathEntity),
    responses(
        (status = OK, body = TrackRead, description = "Successful request"),
        (status = NOT_FOUND, body = inline(APIError), description = "Track ID does not exist in database"),
        (status = INTERNAL_SERVER_ERROR, body = inline(APIError), description = "Server error while handling the request"),
    )
)]
async fn info_track(Path(params_path): Path<PathEntity>) -> Result<(StatusCode, Json<TrackRead>), MalojaError> {
    let result = database::repository::track_info(params_path.id).await?;
    Ok((StatusCode::OK, Json(result)))
}

#[utoipa::path(
    get,
    path = "/album/{id}",
    params(PathEntity),
    responses(
        (status = OK, body = AlbumRead, description = "Successful request"),
        (status = NOT_FOUND, body = inline(APIError), description = "Album ID does not exist in database"),
        (status = INTERNAL_SERVER_ERROR, body = inline(APIError), description = "Server error while handling the request"),
    )
)]
async fn info_album(Path(params_path): Path<PathEntity>) -> Result<(StatusCode, Json<AlbumRead>), MalojaError> {
    let result = database::repository::album_info(params_path.id).await?;
    Ok((StatusCode::OK, Json(result)))
}




#[utoipa::path(
    get,
    path = "/charts_tracks",
    params(QueryTimerange, QueryLimitArtist, QueryLimitAlbum),
    responses(
        (status = OK, body = inline(Charts<TrackRead>), description = "Successful request"),
        (status = INTERNAL_SERVER_ERROR, body = inline(APIError), description = "Server error while handling the request"),
    )
)]
async fn charts_tracks(
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
        (status = OK, body = inline(Charts<ArtistRead>), description = "Successful request"),
        (status = INTERNAL_SERVER_ERROR, body = inline(APIError), description = "Server error while handling the request"),
    )
)]
async fn charts_artists(
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
        (status = OK, body = inline(Charts<AlbumRead>), description = "Successful request"),
        (status = INTERNAL_SERVER_ERROR, body = inline(APIError), description = "Server error while handling the request"),
    )
)]
async fn charts_albums(
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


