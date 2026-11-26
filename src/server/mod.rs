mod pages;
mod pages_admin;
pub mod filters;
mod pages_admin_post;
mod web_auth;
mod notifications;

use std::collections::HashMap;
use crate::api::mount_apis;
use axum::routing::{get, post};
use axum::{middleware, Router};
use axum::http::{header, Uri, StatusCode, HeaderValue};
use axum::response::{IntoResponse, Response};
use axum_csp::{CspValue, CspHeaderBuilder, CspDirectiveType};
use tower_http::set_header::SetResponseHeaderLayer;
use rust_embed::RustEmbed;
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

    // STATIC FILES AND SPECIAL
    app = app.fallback(static_handler);

    // SECURITY
    let csp_rules = CspHeaderBuilder::new()
        .add(CspDirectiveType::DefaultSrc, vec![CspValue::SelfSite])
        .add(CspDirectiveType::ScriptSource, vec![CspValue::SelfSite])
        // maybe move all styles out at some point?
        .add(CspDirectiveType::StyleSource, vec![CspValue::UnsafeInline, CspValue::SelfSite])
        .add(CspDirectiveType::StyleSourceAttr, vec![CspValue::UnsafeInline, CspValue::SelfSite])
        // add image domains once image fetch is set up
        .add(CspDirectiveType::ImgSrc, vec![CspValue::SelfSite])
        .add(CspDirectiveType::ObjectSrc, vec![CspValue::None])
        .add(CspDirectiveType::BaseUri, vec![CspValue::None])
        .finish();

    app = app
        .layer(SetResponseHeaderLayer::if_not_present(
            header::CONTENT_SECURITY_POLICY,
            csp_rules,
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            header::X_CONTENT_TYPE_OPTIONS,
            HeaderValue::from_static("nosniff"),
        ));

    app
}

#[derive(RustEmbed, Clone)]
#[folder = "src/web/static/"]
struct StaticFiles;

#[derive(RustEmbed, Clone)]
#[folder = "src/web/special/"]
struct SpecialFiles;

async fn static_handler(uri: Uri) -> Response {
    let path = uri.path().trim_start_matches('/');
    match StaticFiles::get(path) {
        Some(f) => {
            let mime = f.metadata.mimetype().to_owned();
            ([(header::CONTENT_TYPE, mime)], f.data).into_response()
        }
        None => {
            // we could use simple .html append or smth but this is meant
            // for specific hardcoded paths that need to be explicitly spelled out
            let embedded = match path {
                "api_explorer" => SpecialFiles::get("api_explorer.html"),
                _ => None
            };
            if let Some(f) = embedded {
                let mime = f.metadata.mimetype().to_owned();
                ([(header::CONTENT_TYPE, mime)], f.data).into_response()
            } else {
                StatusCode::NOT_FOUND.into_response()
            }
        },
    }
}

