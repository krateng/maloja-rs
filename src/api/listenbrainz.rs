use axum::Json;
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use crate::api::ScrobbleAPI;

pub const API: ScrobbleAPI = ScrobbleAPI {
    prefix: "/listenbrainz/1",
    tag: "Listenbrainz",
    //endpoints: routes!(submit, validate),
    register: register_routes,
};

fn register_routes(mut router: OpenApiRouter) -> OpenApiRouter {
    router = router.routes(routes!(submit, validate));
    router
}

#[derive(OpenApi)]
#[openapi(
    paths(submit, validate),
    info(title = "Listenbrainz API", version = "1")
)]
pub struct ApiDoc;

#[utoipa::path(
    post,
    path = "/submit-listens",
    responses((status = OK))
)]
pub async fn submit() -> Json<i32> {
    Json(3)
}

#[utoipa::path(
    get,
    path = "/validate-token",
    responses((status = OK))
)]
pub async fn validate() -> Json<i32> {
    Json(3)
}
