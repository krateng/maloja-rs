mod pages;
mod pages_admin;
pub mod filters;
mod pages_admin_post;
mod web_auth;
mod notifications;

use crate::api::mount_apis;
use axum::routing::{get, post};
use axum::{middleware, Router};
use axum::extract::{Request, State};
use axum::middleware::Next;
use axum::response::Response;
use axum_extra::extract::CookieJar;
use pages::*;
use pages_admin::*;
use pages_admin_post::*;
use tower_http::services::{ServeDir, ServeFile};
use crate::configuration::app_state::AppState;
use crate::server::web_auth::require_web_auth;

pub async fn run_server(state: &AppState) -> Result<(), std::io::Error> {
    // we pass state here only for the weird middleware, not yet to finish the app
    let app = create_app(state).await;
    let bind_address = format!("{}:{}", state.config.bind_address, state.config.port);
    let listener = tokio::net::TcpListener::bind(bind_address).await?;
    let stateful_app = app.with_state(state.clone());
    axum::serve(listener, stateful_app).await?;
    Ok(())
}

pub async fn create_app(state: &AppState) -> Router<AppState> {
    // TODO: files in package
    let mut app: Router<AppState> = Router::new();
    // APIS
    app = mount_apis(app);
    // SPECIAL PATHS
    app = app.nest_service("/api_explorer", ServeFile::new("src/web/special/api_explorer.html"));
    // TEMPLATES
    let app_secure = Router::new()
        .route("/admin_auth", get(admin_auth).post(post_admin_auth))
        .route_layer(middleware::from_fn_with_state(state.clone(), require_web_auth));
    app = app
        .route("/about", get(about))
        .route("/artist/{id}", get(info_artist))
        .route("/track/{id}", get(info_track))
        .route("/album/{id}", get(info_album))
        .route("/pulse", get(pulse))
        .route("/performance", get(performance))
        .route("/scrobbles", get(scrobbles))
        .route("/login", get(web_auth::login).post(web_auth::post_login))
        .merge(app_secure);

    // STATIC FILES
    app = app.fallback_service(ServeDir::new("src/web/static"));

    app
}

