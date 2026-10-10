use http_body_util::BodyExt;
use crate::tests::common;
use axum::{body::Body, http::{Request, StatusCode}, Router};
use log::debug;
use serde_json::Value;
use crate::server::{create_app, run_server};
use tower::{Service, ServiceExt};
use crate::tests::fixtures::fixture;

const SERVER_ADDRESS: &str = "http://localhost:42010/apis/maloja_2";

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
    common().await;

    fixture(5, 5, 3).await;

    let app = create_app().await;
    let result = get_json_response(app, "/scrobbles").await;
    assert_eq!(result["pagination"]["items_total"], 3);
}