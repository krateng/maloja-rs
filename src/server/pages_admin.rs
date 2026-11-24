use time::Duration;
use askama::Template;
use axum::extract::{Path, Query, State};
use axum::Form;
use axum::http::HeaderValue;
use axum::response::{Html, IntoResponse, Response};
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use reqwest::header::{COOKIE, LOCATION, SET_COOKIE, WWW_AUTHENTICATE};
use reqwest::{StatusCode, Url};
use serde::Deserialize;
use utoipa::openapi::security::Password;
use crate::AppState;
use crate::entity::ClientModel;
use super::filters;

#[derive(Template)]
#[template(path = "admin_auth.html")]
pub struct AdminAuthPage{
    sessions: Vec<ClientModel>,
    api_keys: Vec<ClientModel>,
    imported: Vec<ClientModel>,
}

pub async fn admin_auth(
    State(state): State<AppState>,
) -> Response {
    let clients = state.db.get_clients().await.unwrap();
    let p = AdminAuthPage {
        sessions: clients.sessions,
        api_keys: clients.api_keys,
        imported: clients.imported,
    };
    Html(p.render().unwrap()).into_response()
}