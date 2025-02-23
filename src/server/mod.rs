mod pages;

use axum::response::{Html, IntoResponse, Response};
use axum::{Json, Router};
use tower_http::services::{ServeDir, ServeFile};
use axum::routing::get;
use crate::api::mount_apis;
use crate::configuration::CONFIG;
use pages::*;

pub async fn run_server() {
    // TODO: files in package

    let mut app = Router::new();
    // APIS
    app = mount_apis(app);
    // SPECIAL PATHS
    app = app
        .nest_service("/api_explorer", ServeFile::new("src/web/special/api_explorer.html"));
    // TEMPLATES
    app = app
        .route("/about", get(about))
        .route("/artist/{id}", get(info_artist))
        .route("/track/{id}", get(info_track))
        .route("/album/{id}", get(info_album));
    // STATIC FILES
    app = app.fallback_service(ServeDir::new("src/web/static"));

    let bind_address = format!("{}:{}", CONFIG.bind_address, CONFIG.port);
    let listener = tokio::net::TcpListener::bind(bind_address).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
