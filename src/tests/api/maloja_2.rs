use std::fs::read;
use crate::server::create_app;
use crate::tests::common;
use crate::tests::fixtures::fixture;
use axum::{
    body::Body,
    http::Request,
    Router,
};
use http_body_util::BodyExt;
use serde_json::Value;
use tower::ServiceExt;
use crate::AppState;

const SERVER_ADDRESS: &str = "http://localhost:42010/apis/maloja_2";

async fn setup_test_router(state: &AppState) -> Router {
    let router = create_app().await;
    router.with_state(state.clone())
}

async fn get_json_response(app: Router, path: &str) -> Value {
    let url = format!("{}{}", SERVER_ADDRESS, path);
    let request = Request::get(url).body(Body::empty()).unwrap();
    let response = app.oneshot(request).await.unwrap();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let data: Value = serde_json::from_slice(&bytes).unwrap();
    data
}

#[tokio::test]
async fn get_scrobbles_all() {
    let state = common().await;
    let app = setup_test_router(&state).await;

    fixture(5, 5, 3, &state).await;

    let result = get_json_response(app, "/scrobbles").await;
    assert_eq!(result["pagination"]["items_total"], 3);
}
