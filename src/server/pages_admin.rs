use time::Duration;
use askama::Template;
use axum::extract::{Path, Query, State};
use axum::http::HeaderValue;
use axum::response::{Html, IntoResponse, Response};
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use reqwest::header::{COOKIE, LOCATION, SET_COOKIE, WWW_AUTHENTICATE};
use reqwest::{StatusCode, Url};
use serde::Deserialize;
use crate::AppState;
use crate::entity::ClientModel;
use super::filters;


// LOGIN

#[derive(Template)]
#[template(path = "login.html")]
struct LoginPage{
    failed: bool,
    sessions: Vec<ClientModel>
}

#[derive(Deserialize)]
pub struct QueryLogin {
    target: Option<String>
}

pub async fn login(
    State(state): State<AppState>,
) -> Response {
    let clients = state.db.get_clients().await.unwrap();
    let p = LoginPage {
        failed: false,
        sessions: clients.sessions,
    };
    Html(p.render().unwrap()).into_response()
}
pub async fn post_login(
    State(state): State<AppState>,
    Query(params_login): Query<QueryLogin>
) -> Response {

    let correct = true; //TODO

    if correct {
        let target = match params_login.target {
            Some(target) if is_safe_target(&target) => target,
            _ => "/".into()
        };
        let r = axum::response::Redirect::to(&target);

        let jar = CookieJar::new();
        let cookie = Cookie::build(("session", "wat"))
            .http_only(true)
            .secure(true) // need to drop if anyone uses this without ssl rev proxy
            .same_site(SameSite::Strict)
            .path("/")
            .max_age(Duration::days(30));
        (jar.add(cookie), r).into_response()
    }
    else {
        let clients = state.db.get_clients().await.unwrap();
        let p = LoginPage {
            failed: true,
            sessions: clients.sessions,
        };
        let mut r = Html(p.render().unwrap()).into_response();
        r.headers_mut().insert(WWW_AUTHENTICATE, HeaderValue::from_static("Cookie"));
        *r.status_mut() = StatusCode::UNAUTHORIZED;
        r
    }

}

fn is_safe_target(target: &str) -> bool {
    let b = target.as_bytes();
    b.first() == Some(&b'/') && b.get(1).is_none_or(u8::is_ascii_lowercase)
}

// BACKEND


#[derive(Template)]
#[template(path = "admin_auth.html")]
struct AdminAuthPage{
    sessions: Vec<ClientModel>,
    api_keys: Vec<ClientModel>
}

pub async fn admin_auth(
    State(state): State<AppState>,
) -> Response {
    let clients = state.db.get_clients().await.unwrap();
    let p = AdminAuthPage {
        sessions: clients.sessions,
        api_keys: clients.api_keys,
    };
    Html(p.render().unwrap()).into_response()
}