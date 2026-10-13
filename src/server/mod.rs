mod pages;

use crate::api::mount_apis;
use crate::configuration::CONFIG;
use axum::routing::get;
use axum::Router;
use pages::*;
use tower_http::services::{ServeDir, ServeFile};

pub async fn run_server() -> Result<(), std::io::Error> {
    let app = create_app().await;
    let bind_address = format!("{}:{}", CONFIG.bind_address, CONFIG.port);
    let listener = tokio::net::TcpListener::bind(bind_address).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

pub async fn create_app() -> Router {
    // TODO: files in package

    let mut app = Router::new();
    // APIS
    app = mount_apis(app);
    // SPECIAL PATHS
    app = app.nest_service("/api_explorer", ServeFile::new("src/web/special/api_explorer.html"));
    // TEMPLATES
    app = app
        .route("/about", get(about))
        .route("/artist/{id}", get(info_artist))
        .route("/track/{id}", get(info_track))
        .route("/album/{id}", get(info_album))
        .route("/pulse", get(pulse))
        .route("/performance", get(performance))
        .route("/scrobbles", get(scrobbles));
    // STATIC FILES
    app = app.fallback_service(ServeDir::new("src/web/static"));

    app
}
