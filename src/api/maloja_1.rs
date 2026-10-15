use crate::api::ScrobbleAPI;
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use crate::configuration::app_state::AppState;

pub const API: ScrobbleAPI = ScrobbleAPI {
    prefix: "/maloja_1",
    tag: "Maloja v1 (Compatibility)",
    register: register_routes,
};

fn register_routes(mut router: OpenApiRouter<AppState>) -> OpenApiRouter<AppState> {
    router = router.routes(routes!(scrobble));
    router
}

#[derive(OpenApi)]
#[openapi(paths(scrobble), info(title = "Maloja API", version = "1 (Partial implementation)"))]
pub struct ApiDoc;

#[utoipa::path(
    get,
    path = "/scrobble",
    responses(
        (status = OK, body = (), description = "Successful request"),
    )
)]
async fn scrobble() {}
