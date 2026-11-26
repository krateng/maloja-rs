use axum::extract::path::ErrorKind;
use axum::extract::rejection::PathRejection;
use axum::extract::{FromRequestParts, Query, State};
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::any;
use axum::Json;
use serde::de::DeserializeOwned;
use serde::Serialize;
use utoipa::{OpenApi, ToSchema};
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use maloja_dto::api::v2::post::{
    EmbeddedAlbumWrite,
    EmbeddedContextArtistWrite,
    EmbeddedTrackWrite,
    ScrobbleWrite,
};
use maloja_dto::api::v2::get::{
    Artist, Album, Track, ArtistContext, Scrobble,
    Paginated,
    TopEntry as APITopEntry, ChartsEntry as APIChartsEntry,
    PulseEntry as APIPulseEntry, PerformanceEntry as APIPerformanceEntry,
};

use crate::api::ScrobbleAPI;
use crate::configuration::app_state::AppState;
use crate::database::errors::MalojaError;
use crate::database::repository::get_or_create::AttributedScrobbleWrite;
use crate::database::views::{ChartsEntry, PerformanceEntry, PulseEntry, TopEntry};
use crate::entity::{AlbumRead, ArtistRead, TrackRead, ScrobbleRead, ScrobbleWrite as InternalScrobbleWrite, TrackWrite, AlbumWrite, ArtistWriteContext, ArtistReadContext};
use crate::uri::{
    PathEntity, QueryLimitAlbum, QueryLimitArtist, QueryLimitTrack, QueryPagination,
    QueryTimerange, QueryTimesteps,
};

pub const API: ScrobbleAPI =
    ScrobbleAPI { prefix: "/maloja_2", tag: "Maloja v2", register: register_routes };

fn register_routes(mut router: OpenApiRouter<AppState>) -> OpenApiRouter<AppState> {
    router = router
        .routes(routes!(info_artist))
        .routes(routes!(info_track))
        .routes(routes!(info_album))
        .routes(routes!(charts_tracks))
        .routes(routes!(charts_artists))
        .routes(routes!(charts_albums))
        .routes(routes!(scrobbles))
        .routes(routes!(pulse))
        .routes(routes!(performance))
        .routes(routes!(submit_scrobbles))
        //.fallback(notfound); // TODO: https://github.com/tokio-rs/axum/issues/3138
        .route("/{*rest}", any(notfound));
    router
}

#[derive(OpenApi)]
#[openapi(
    paths(
        charts_tracks,
        charts_artists,
        charts_albums,
        info_artist,
        info_album,
        info_track,
        scrobbles,
        pulse,
        performance,
        submit_scrobbles,
    ),
    info(title = "Maloja API", version = "2"),
    components(schemas(Scrobble, Track, Artist, Album))
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
            description,
        }),
    )
        .into_response()
}

impl IntoResponse for MalojaError {
    fn into_response(self) -> Response {
        match &self {
            MalojaError::ArtistNotFound { id } => {
                create_response(&self, StatusCode::NOT_FOUND, format!("Artist {} not found", id))
            }
            MalojaError::TrackNotFound { id } => {
                create_response(&self, StatusCode::NOT_FOUND, format!("Track {} not found", id))
            }
            MalojaError::AlbumNotFound { id } => {
                create_response(&self, StatusCode::NOT_FOUND, format!("Album {} not found", id))
            }
            MalojaError::EntityNotFound { id } => {
                create_response(&self, StatusCode::NOT_FOUND, format!("Entity {} not found", id))
            }
            MalojaError::DatabaseError { message } => {
                create_response(&self, StatusCode::INTERNAL_SERVER_ERROR, message.clone())
            }
            MalojaError::DatabaseConnectionError { message } => {
                create_response(&self, StatusCode::INTERNAL_SERVER_ERROR, message.clone())
            }
            MalojaError::ParseError { message } => {
                create_response(&self, StatusCode::BAD_REQUEST, message.clone())
            }
            MalojaError::TimeError { message } => {
                create_response(&self, StatusCode::INTERNAL_SERVER_ERROR, message.clone())
            }
            MalojaError::QueryError { message } => {
                create_response(&self, StatusCode::BAD_REQUEST, message.clone())
            }
            MalojaError::MissingField { field } => {
                create_response(&self, StatusCode::BAD_REQUEST, format!("Missing field: {}", field.clone()))
            }
            MalojaError::FilesystemError { message } => {
                create_response(&self, StatusCode::INTERNAL_SERVER_ERROR, message.clone())
            }
            MalojaError::InternalError { message } => {
                create_response(&self, StatusCode::INTERNAL_SERVER_ERROR, message.clone())
            }
            MalojaError::PasswordError => {
                create_response(&self, StatusCode::BAD_REQUEST, String::from("Passwords must be at least 12 characters"))
            }
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
                            ErrorKind::ParseErrorAtKey { key, .. } => {
                                format!("Key <{}> could not be parsed", key)
                            }
                            _ => "Parameters could not be parsed".to_string(),
                        };
                        (
                            status,
                            APIError {
                                error_type: "RequestParamsParseError".to_string(),
                                description: description.to_string(),
                            },
                        )
                    }
                    PathRejection::MissingPathParams(_error) => (
                        StatusCode::BAD_REQUEST,
                        APIError {
                            error_type: "RequestParamsParseError".to_string(),
                            description: "Missing path parameters".to_string(),
                        },
                    ),
                    _ => (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        APIError {
                            error_type: "RequestParamsParseError".to_string(),
                            description: "Unknown error while parsing path parameters".to_string(),
                        },
                    ),
                };

                Err((status, Json(body)))
            }
        }
    }
}

pub async fn notfound() -> Response {
    (
        StatusCode::NOT_FOUND,
        Json(APIError {
            error_type: "InvalidEndpoint".to_string(),
            description: "Endpoint does not exist in this API.".to_string(),
        }),
    )
        .into_response()
}

#[utoipa::path(
    get,
    path = "/artist/{id}",
    params(PathEntity),
    responses(
        (status = OK, body = Artist, description = "Successful request"),
        (status = NOT_FOUND, body = inline(APIError), description = "Artist ID does not exist in database"),
        (status = INTERNAL_SERVER_ERROR, body = inline(APIError), description = "Server error while handling the request"),
    )
)]
async fn info_artist(
    State(state): State<AppState>,
    Path(params_path): Path<PathEntity>,
) -> Result<(StatusCode, Json<Artist>), MalojaError> {
    let result = state.db.artist_info(params_path.id).await?;
    Ok((StatusCode::OK, Json(result.into())))
}

#[utoipa::path(
    get,
    path = "/track/{id}",
    params(PathEntity),
    responses(
        (status = OK, body = Track, description = "Successful request"),
        (status = NOT_FOUND, body = inline(APIError), description = "Track ID does not exist in database"),
        (status = INTERNAL_SERVER_ERROR, body = inline(APIError), description = "Server error while handling the request"),
    )
)]
async fn info_track(
    State(state): State<AppState>,
    Path(params_path): Path<PathEntity>,
) -> Result<(StatusCode, Json<Track>), MalojaError> {
    let result = state.db.track_info(params_path.id).await?;
    Ok((StatusCode::OK, Json(result.into())))
}

#[utoipa::path(
    get,
    path = "/album/{id}",
    params(PathEntity),
    responses(
        (status = OK, body = Album, description = "Successful request"),
        (status = NOT_FOUND, body = inline(APIError), description = "Album ID does not exist in database"),
        (status = INTERNAL_SERVER_ERROR, body = inline(APIError), description = "Server error while handling the request"),
    )
)]
async fn info_album(
    State(state): State<AppState>,
    Path(params_path): Path<PathEntity>,
) -> Result<(StatusCode, Json<Album>), MalojaError> {
    let result = state.db.album_info(params_path.id).await?;
    Ok((StatusCode::OK, Json(result.into())))
}

#[utoipa::path(
    get,
    path = "/charts_tracks",
    params(QueryTimerange, QueryLimitArtist, QueryLimitAlbum, QueryPagination),
    responses(
        (status = OK, body = inline(Paginated<APIChartsEntry<Track>>), description = "Successful request"),
        (status = BAD_REQUEST, body = inline(APIError), description = "Error in request"),
        (status = INTERNAL_SERVER_ERROR, body = inline(APIError), description = "Server error while handling the request"),
    )
)]
async fn charts_tracks(
    State(state): State<AppState>,
    Query(params_time): Query<QueryTimerange>,
    Query(params_limit_artist): Query<QueryLimitArtist>,
    Query(params_limit_album): Query<QueryLimitAlbum>,
    Query(params_pagination): Query<QueryPagination>,
) -> Result<(StatusCode, Json<Paginated<APIChartsEntry<Track>>>), MalojaError> {
    let timerange = params_time.to_timerange()?;
    let artist_id = params_limit_artist.to_artist_id();
    let album_id = params_limit_album.to_album_id();
    let tracks = state.db.charts_tracks(&timerange, artist_id, album_id).await?;
    let tracks_api = tracks.into_iter().map(|t| t.into()).collect();
    let paginated = params_pagination.paginate_results(tracks_api);
    Ok((
        StatusCode::OK,
        Json(paginated),
    ))
}

#[utoipa::path(
    get,
    path = "/charts_artists",
    params(QueryTimerange, QueryPagination),
    responses(
        (status = OK, body = inline(Paginated<APIChartsEntry<Artist>>), description = "Successful request"),
        (status = BAD_REQUEST, body = inline(APIError), description = "Error in request"),
        (status = INTERNAL_SERVER_ERROR, body = inline(APIError), description = "Server error while handling the request"),
    )
)]
async fn charts_artists(
    State(state): State<AppState>,
    Query(params_time): Query<QueryTimerange>,
    Query(params_pagination): Query<QueryPagination>,
) -> Result<(StatusCode, Json<Paginated<APIChartsEntry<Artist>>>), MalojaError> {
    let timerange = params_time.to_timerange()?;
    let artists = state.db.charts_artists(&timerange).await?;
    let artists_api = artists.into_iter().map(|a| a.into()).collect();
    let paginated = params_pagination.paginate_results(artists_api);
    Ok((
        StatusCode::OK,
        Json(paginated),
    ))
}

#[utoipa::path(
    get,
    path = "/charts_albums",
    params(QueryTimerange, QueryLimitArtist, QueryPagination),
    responses(
        (status = OK, body = inline(Paginated<APIChartsEntry<Album>>), description = "Successful request"),
        (status = BAD_REQUEST, body = inline(APIError), description = "Error in request"),
        (status = INTERNAL_SERVER_ERROR, body = inline(APIError), description = "Server error while handling the request"),
    )
)]
async fn charts_albums(
    State(state): State<AppState>,
    Query(params_time): Query<QueryTimerange>,
    Query(params_limit_artist): Query<QueryLimitArtist>,
    Query(params_pagination): Query<QueryPagination>,
) -> Result<(StatusCode, Json<Paginated<APIChartsEntry<Album>>>), MalojaError> {
    let timerange = params_time.to_timerange()?;
    let artist_id = params_limit_artist.to_artist_id();
    let albums = state.db.charts_albums(&timerange, artist_id).await?;
    let albums_api = albums.into_iter().map(|a| a.into()).collect();
    let paginated_albums = params_pagination.paginate_results(albums_api);
    Ok((
        StatusCode::OK,
        Json(paginated_albums),
    ))
}

#[utoipa::path(
    get,
    path = "/scrobbles",
    params(QueryTimerange, QueryLimitArtist, QueryLimitAlbum, QueryLimitTrack, QueryPagination),
    responses(
        (status = OK, body = inline(Paginated<Scrobble>), description = "Successful request"),
        (status = BAD_REQUEST, body = inline(APIError), description = "Error in request"),
        (status = INTERNAL_SERVER_ERROR, body = inline(APIError), description = "Server error while handling the request"),
    )
)]
async fn scrobbles(
    State(state): State<AppState>,
    Query(params_time): Query<QueryTimerange>,
    Query(params_limit_artist): Query<QueryLimitArtist>,
    Query(params_limit_album): Query<QueryLimitAlbum>,
    Query(params_limit_track): Query<QueryLimitTrack>,
    Query(params_pagination): Query<QueryPagination>,
) -> Result<(StatusCode, Json<Paginated<Scrobble>>), MalojaError> {
    let timerange = params_time.to_timerange()?;
    let artist_id = params_limit_artist.to_artist_id();
    let album_id = params_limit_album.to_album_id();
    let track_id = params_limit_track.to_track_id();
    let scrobbles =
        state.db.scrobbles(&timerange, artist_id, album_id, track_id, true).await?;
    let scrobbles_api = scrobbles.into_iter().map(|s| s.into()).collect();
    let paginated_scrobbles = params_pagination.paginate_results(scrobbles_api);
    Ok((StatusCode::OK, Json(paginated_scrobbles)))
}

#[utoipa::path(
    get,
    path = "/pulse",
    params(QueryTimerange, QueryTimesteps, QueryLimitArtist, QueryLimitAlbum, QueryLimitTrack, QueryPagination),
    responses(
        (status = OK, body = inline(Paginated<APIPulseEntry>), description = "Successful request"),
        (status = BAD_REQUEST, body = inline(APIError), description = "Error in request"),
        (status = INTERNAL_SERVER_ERROR, body = inline(APIError), description = "Server error while handling the request"),
    )
)]
async fn pulse(
    State(state): State<AppState>,
    Query(params_time): Query<QueryTimerange>,
    Query(params_timesteps): Query<QueryTimesteps>,
    Query(params_limit_artist): Query<QueryLimitArtist>,
    Query(params_limit_album): Query<QueryLimitAlbum>,
    Query(params_limit_track): Query<QueryLimitTrack>,
    Query(params_pagination): Query<QueryPagination>,
) -> Result<(StatusCode, Json<Paginated<APIPulseEntry>>), MalojaError> {
    let timerange = params_time.to_timerange()?;
    let subranges = timerange.get_subranges(params_timesteps.to_rangetype()?);
    let artist_id = params_limit_artist.to_artist_id();
    let album_id = params_limit_album.to_album_id();
    let track_id = params_limit_track.to_track_id();

    let result = state.db.pulse(subranges, artist_id, album_id, track_id).await?;
    let result_api = result.into_iter().map(|e| e.into()).collect();
    let paginated_pulse = params_pagination.paginate_results(result_api);
    Ok((StatusCode::OK, Json(paginated_pulse)))
}

#[utoipa::path(
    get,
    path = "/performance",
    params(QueryTimerange, QueryTimesteps, QueryLimitArtist, QueryLimitAlbum, QueryLimitTrack, QueryPagination),
    responses(
        (status = OK, body = inline(Paginated<APIPerformanceEntry>), description = "Successful request"),
        (status = BAD_REQUEST, body = inline(APIError), description = "Error in request"),
        (status = INTERNAL_SERVER_ERROR, body = inline(APIError), description = "Server error while handling the request"),
    )
)]
async fn performance(
    State(state): State<AppState>,
    Query(params_time): Query<QueryTimerange>,
    Query(params_timesteps): Query<QueryTimesteps>,
    Query(params_limit_artist): Query<QueryLimitArtist>,
    Query(params_limit_album): Query<QueryLimitAlbum>,
    Query(params_limit_track): Query<QueryLimitTrack>,
    Query(params_pagination): Query<QueryPagination>,
) -> Result<(StatusCode, Json<Paginated<APIPerformanceEntry>>), MalojaError> {
    let timerange = params_time.to_timerange()?;
    let subranges = timerange.get_subranges(params_timesteps.to_rangetype()?);
    let artist_id = params_limit_artist.to_artist_id();
    let album_id = params_limit_album.to_album_id();
    let track_id = params_limit_track.to_track_id();

    let result =
        state.db.performance(subranges, artist_id, album_id, track_id).await?;
    let result_api = result.into_iter().map(|e| e.into()).collect();
    let paginated_performance = params_pagination.paginate_results(result_api);
    Ok((StatusCode::OK, Json(paginated_performance)))
}


#[utoipa::path(
    post,
    path = "/scrobble",
    request_body = Vec<ScrobbleWrite>,
    responses(
        (status = OK, body = inline(Vec<Scrobble>), description = "Successful scrobble"),
        (status = BAD_REQUEST, body = inline(APIError), description = "Error in request"),
        (status = INTERNAL_SERVER_ERROR, body = inline(APIError), description = "Server error while handling the request"),
    )
)]
async fn submit_scrobbles(
    State(state): State<AppState>,
    Json(scrobbles): Json<Vec<ScrobbleWrite>>
) -> Result<(StatusCode, Json<Vec<Scrobble>>), MalojaError> {
    // TODO proper client identification
    let scrobbles: Vec<AttributedScrobbleWrite> = scrobbles.into_iter().map(|s| AttributedScrobbleWrite { write: s.into(), client_id: None }).collect();
    let result = state.db.create_scrobbles_return_reads(scrobbles, true, true).await?;
    let result_api = result.into_iter().map(|s| s.into()).collect();
    Ok((StatusCode::OK, Json(result_api)))
}



// transformations between internal read/write structs and api definitions
// a lot of boilerplate for what is exactly the same struct, but we want the
// api definitions fixed outside the crate and not use them internally

impl From<ScrobbleWrite> for InternalScrobbleWrite {
    fn from(scrobble: ScrobbleWrite) -> Self {
        InternalScrobbleWrite {
            timestamp: scrobble.timestamp,
            track: scrobble.track.into(),
            listen_duration: scrobble.listen_duration,
        }
    }
}

impl From<EmbeddedTrackWrite> for TrackWrite {
    fn from(track: EmbeddedTrackWrite) -> Self {
        TrackWrite {
            id: track.id,
            title: track.title,
            artists: track.artists.map(|va| va.into_iter().map(|a| a.into()).collect()),
            track_length: track.track_length,
            album: track.album.map(|a| a.into()),
            mbid: track.mbid,
            spotify_id: track.spotify_id,
        }
    }
}

impl From<EmbeddedAlbumWrite> for AlbumWrite {
    fn from(album: EmbeddedAlbumWrite) -> Self {
        AlbumWrite {
            id: album.id,
            album_title: album.album_title,
            album_artists: album.album_artists.map(|va| va.into_iter().map(|a| a.into()).collect()),
            mbid: album.mbid,
            spotify_id: album.spotify_id,
        }
    }
}

impl From<EmbeddedContextArtistWrite> for ArtistWriteContext {
    fn from(artist: EmbeddedContextArtistWrite) -> Self {
        ArtistWriteContext {
            id: artist.id,
            name: artist.name,
            alias: artist.alias,
            primary: artist.primary,
            mbid: artist.mbid,
            spotify_id: artist.spotify_id,
        }
    }
}

impl From<ScrobbleRead> for Scrobble {
    fn from(scrobble: ScrobbleRead) -> Self {
        Scrobble {
            timestamp: scrobble.timestamp,
            time_local: scrobble.time_local,
            track: scrobble.track.into(),
        }
    }
}

impl From<TrackRead> for Track {
    fn from(track: TrackRead) -> Self {
        Track {
            id: track.id,
            title: track.title,
            artists: track.artists.into_iter().map(|a| a.into()).collect(),
            album: track.album.map(|a| a.into() ),
            track_length: track.track_length,
        }
    }
}

impl From<AlbumRead> for Album {
    fn from(album: AlbumRead) -> Self {
        Album {
            id: album.id,
            album_title: album.album_title,
            album_artists: album.album_artists.into_iter().map(|a| a.into()).collect(),
        }
    }
}

impl From<ArtistReadContext> for ArtistContext {
    fn from(artist: ArtistReadContext) -> Self {
        ArtistContext {
            id: artist.id,
            name: artist.name,
            alias: artist.alias,
            primary: artist.primary,
        }
    }
}

impl From<ArtistRead> for Artist {
    fn from(artist: ArtistRead) -> Self {
        Artist {
            id: artist.id,
            name: artist.name,
        }
    }
}

impl<I, O> From<TopEntry<I>> for APITopEntry<O>
where O: From<I>
{
    fn from(value: TopEntry<I>) -> Self {
        APITopEntry {
            time_range: value.time_range.describe_short(),
            entry: value.entry.into(),
            scrobbles: value.scrobbles,
        }
    }
}

impl<I, O> From<ChartsEntry<I>> for APIChartsEntry<O>
where O: From<I>
{
    fn from(value: ChartsEntry<I>) -> Self {
        APIChartsEntry {
            rank: value.rank as u32,
            entry: value.entry.into(),
            scrobbles: value.scrobbles,
        }
    }
}

impl From<PulseEntry> for APIPulseEntry {
    fn from(value: PulseEntry) -> Self {
        APIPulseEntry {
            time_range: value.time_range.describe_short(),
            scrobbles: value.scrobbles,
        }
    }
}

impl From<PerformanceEntry> for APIPerformanceEntry {
    fn from(value: PerformanceEntry) -> Self {
        APIPerformanceEntry {
            time_range: value.time_range.describe_short(),
            rank: value.rank,
        }
    }
}