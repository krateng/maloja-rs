use time::Duration;
use askama::Template;
use axum::extract::{Path, Query, State};
use axum::Form;
use axum::http::HeaderValue;
use axum::response::{Html, IntoResponse, Response};
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use crate::AppState;
use crate::entity::ClientModel;
use crate::server::notifications::{Notification, Toast};
use crate::server::pages::SharedPage;
use super::filters;

#[derive(Template)]
#[template(path = "admin_auth.html")]
pub struct AdminAuthPage{
    sessions: Vec<ClientModel>,
    api_keys: Vec<ClientModel>,
    imported: Vec<ClientModel>,
    shared_page: SharedPage,
}

pub async fn admin_auth(
    jar: CookieJar,
    State(state): State<AppState>,
) -> Response {
    let clients = state.db.get_clients().await.unwrap();
    let p = AdminAuthPage {
        sessions: clients.sessions,
        api_keys: clients.api_keys,
        imported: clients.imported,
        shared_page: SharedPage {
            notifications: extract_flash_cookies_full(jar)
        },
    };
    Html(p.render().unwrap()).into_response()
}

fn extract_flash_cookie(jar: CookieJar) -> Option<Toast> {
    let cookie = jar.get("flash_message");
    if let Some(cookie) = cookie
        && let Some(notif) = Notification::from_cookie(&cookie.value()) {
        Some(notif.into_toast())
    }
    else {
        println!("cookie not found: {:?}", cookie);
        None
    }
}
fn extract_flash_cookies_full(jar: CookieJar) -> Vec<Toast> {
    let toast = extract_flash_cookie(jar);
    if let Some(toast) = toast {
        vec![toast]
    }
    else {
        Vec::new()
    }
}