use axum::Router;
use tower_http::services::{ServeDir, ServeFile};

use crate::api::mount_apis;
use crate::configuration::CONFIG;

pub async fn run_server() {
    let mut app = Router::new();

    app = mount_apis(app);
    // TODO: files in package
    app = app
        .nest_service("/api_explorer", ServeFile::new("src/web/special/api_explorer.html"));
    app = app.fallback_service(ServeDir::new("src/web/static"));

    let bind_address = format!("{}:{}", CONFIG.bind_address, CONFIG.port);
    let listener = tokio::net::TcpListener::bind(bind_address).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
