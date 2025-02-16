use axum::routing::get;
use axum::{Json, Router};
use axum::http::header;
use axum::response::{IntoResponse, Response};
use serde_json::json;
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;

mod audioscrobbler;
mod listenbrainz;
mod maloja_2;

const APIS: [ScrobbleAPI; 3] = [listenbrainz::API, audioscrobbler::API, maloja_2::API];

pub struct ScrobbleAPI {
    pub prefix: &'static str,
    pub tag: &'static str,
    //endpoints: UtoipaMethodRouter,
    register: fn(OpenApiRouter) -> OpenApiRouter,
}
impl ScrobbleAPI {
    //pub fn register_routes(&self) -> OpenApiRouter {
    //    let mut router = OpenApiRouter::new();
    //    router = router.routes(self.endpoints.clone());
    //    router
    //}
}

#[derive(OpenApi)]
#[openapi(
    paths(),
    nest(
        (path = listenbrainz::API.prefix, api = listenbrainz::ApiDoc, tags = [listenbrainz::API.tag]),
        (path = audioscrobbler::API.prefix, api = audioscrobbler::ApiDoc, tags = [audioscrobbler::API.tag]),
        (path = maloja_2::API.prefix, api = maloja_2::ApiDoc, tags = [maloja_2::API.tag]),
    ),
    servers(
        (url = "/apis")
    ),
    info(
        title = "Maloja APIs",
        version = "4.0.0",
        description = "You may also refer to the documentation of the <a href='https://www.last.fm/api/scrobbling'>Audioscrobbler API</a> and the <a href='https://listenbrainz.readthedocs.io/en/latest/users/api/'>ListenBrainz API</a>.",
    )
)]
pub struct ApiDoc {}

pub fn mount_apis(root_router: Router) -> Router {
    let mut api_router = OpenApiRouter::new();
    api_router = api_router.route("/openapi.json", get(openapi));
    //let mut api_explorer = SwaggerUi::new("/api_explorer2");

    for api in APIS {
        let mut specific_api_router = OpenApiRouter::new();
        specific_api_router = (api.register)(specific_api_router);
        api_router = api_router.nest(&api.prefix, specific_api_router);
    }

    let (api_router_r, _api_router_oapi) = api_router.split_for_parts();

    let root_router = root_router.nest("/apis", api_router_r);
    //root_router = root_router.merge(api_explorer);
    root_router
}

async fn openapi() -> Response {
    let json_openapi = json!(
        ApiDoc::openapi()
    );
    let mut resp = Json(json_openapi).into_response();
    resp.headers_mut().insert(header::CONTENT_TYPE, "application/json".parse().unwrap());
    resp
}
