use axum::extract::State;
use axum_extra::extract::Query;
use axum::http::StatusCode;
use axum::{http, Json};
use crate::api::ScrobbleAPI;
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use crate::configuration::app_state::AppState;
use maloja_dto::api::v1::{SubmitScrobble, ScrobbleResponse, ScrobbleResponseSuccess, ScrobbleResponseError, ResponseTrack, ResponseWarning, ResponseErrorType, ResponseStatusFailed, ResponseStatusSucceeded, ResponseWarningOrErrorValue, ResponseError};
use crate::database::errors::MalojaError;
use crate::database::repository::get_or_create::AttributedScrobbleWrite;
use crate::entity::{AlbumRead, ArtistRead, TrackRead, ScrobbleRead, ScrobbleWrite as InternalScrobbleWrite, TrackWrite, AlbumWrite, ArtistWriteContext, ArtistReadContext};

pub const API: ScrobbleAPI = ScrobbleAPI {
    prefix: "/maloja_1",
    tag: "Maloja v1 (Compatibility)",
    register: register_routes,
};

fn register_routes(mut router: OpenApiRouter<AppState>) -> OpenApiRouter<AppState> {
    router = router
        .routes(routes!(scrobble));
    router
}

#[derive(OpenApi)]
#[openapi(paths(scrobble), info(title = "Maloja API", version = "1 (Partial implementation)"))]
pub struct ApiDoc;

#[utoipa::path(
    post,
    path = "/scrobble",
    params(SubmitScrobble),
    responses(
        (status = OK, body = inline(ScrobbleResponseSuccess), description = "Successful request"),
        (status = BAD_REQUEST, body = inline(ScrobbleResponseError), description = "Error while scrobbling"),
    )
)]
async fn scrobble(
    State(state): State<AppState>,
    Query(scrobble): Query<SubmitScrobble>
) -> (StatusCode, Json<ScrobbleResponse>) {
    let fix = !scrobble.no_fix;
    let scrobble: InternalScrobbleWrite = scrobble.into();
    let attributed = AttributedScrobbleWrite { write: scrobble, client_id: None };
    let result = state.db.create_scrobbles_return_reads(vec![attributed], true, fix).await;

    match result {
        Ok(result) => {
            if let Some(first) = result.into_iter().next() {
                let artists = first.track.artists.into_iter().map(|a| a.name).collect::<Vec<_>>();
                let title = first.track.title;
                let response = ScrobbleResponseSuccess {
                    status: ResponseStatusSucceeded::Success,
                    track: ResponseTrack {
                        artists: artists.clone(),
                        title: title.clone(),
                    },
                    desc: format!(
                        "Scrobbled {} by {}",
                        title,
                        artists.join(", ")
                    ),
                    warnings: None,
                };
                (StatusCode::OK, Json(ScrobbleResponse::Success(response)))
            }
            else {
                return_error(None)
            }
        }
        Err(err) => {
            return_error(Some(err))
        }
    }
}

fn return_error(err: Option<MalojaError>) -> (StatusCode, Json<ScrobbleResponse>) {
    let response = ScrobbleResponseError {
        status: ResponseStatusFailed::Error,
        error: ResponseError {
            error_type: ResponseErrorType::UnknownPleaseUpgrade,
            value: err.map(|e| ResponseWarningOrErrorValue::Single(e.to_string())) ,
            desc: "This legacy endpoint can no longer give detailed information on errors. Please use the new endpoint /apis/maloja_v2/submit_scrobbles".to_string(),
        }
    };
    (StatusCode::BAD_REQUEST, Json(ScrobbleResponse::Error(response)))
}

impl From<SubmitScrobble> for InternalScrobbleWrite {
    fn from(value: SubmitScrobble) -> Self {
        InternalScrobbleWrite {
            timestamp: value.time,
            track: TrackWrite {
                id: None,
                title: Some(value.title),
                artists: value.artists.map(|va| va.into_iter().map(|a| {
                    ArtistWriteContext {
                        id: None,
                        name: Some(a),
                        alias: None,
                        primary: true,
                        mbid: None,
                        spotify_id: None,
                    }
                }).collect()),
                track_length: value.length,
                album: value.album.map(|a| AlbumWrite {
                    id: None,
                    album_title: Some(a),
                    album_artists: value.album_artists.map(|va| va.into_iter().map(|a| {
                        ArtistWriteContext {
                            id: None,
                            name: Some(a),
                            alias: None,
                            primary: true,
                            mbid: None,
                            spotify_id: None,
                        }
                    }).collect()),
                    mbid: None,
                    spotify_id: None,
                }),
                mbid: None,
                spotify_id: None,
            },
            listen_duration: value.duration,
        }
    }
}

