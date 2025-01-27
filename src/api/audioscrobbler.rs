use axum::Json;
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use crate::api::ScrobbleAPI;

pub const API: ScrobbleAPI = ScrobbleAPI {
    prefix: "/audioscrobbler/2.0",
    tag: "Audioscrobbler (Last.fm)",
    //endpoints: routes!(mainendpoint, mainendpoint_post),
    register: register_routes,
};

fn register_routes(mut router: OpenApiRouter) -> OpenApiRouter {
    router = router.routes(routes!(mainendpoint, mainendpoint_post));
    router
}

#[derive(OpenApi)]
#[openapi(
    paths(mainendpoint),
    info(title = "Audioscrobbler API", version = "2.0")
)]
pub struct ApiDoc;


#[utoipa::path(
    get,
    path = "",
    responses(
        (status = 200, description = "Success!"),
        (status = 418, description = "🫖")
    ),
    summary = "Root Endpoint",
    description = "In accordance with the <a href='https://www.last.fm/api'>specification</a>, this endpoint is used for all operations. The query argument 'method' is used to determine the operation."
)]
pub async fn mainendpoint() -> Json<i32> {
    Json(3)
}

#[utoipa::path(
    post,
    path = "",
    responses(
        (status = 200, description = "Success!"),
        (status = 418, description = "🫖")
    ),
    summary = "Root Endpoint POST",
    description = "In accordance with the <a href='https://www.last.fm/api'>specification</a>, this endpoint is used for all operations. The query argument 'method' is used to determine the operation."
)]
pub async fn mainendpoint_post() -> Json<i32> {
    Json(42)
}

