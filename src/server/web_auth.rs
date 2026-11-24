use askama::Template;
use axum::extract::{OriginalUri, Query, Request, State};
use axum::Form;
use axum::http::header::WWW_AUTHENTICATE;
use axum::http::{HeaderValue, StatusCode};
use axum::middleware::Next;
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum_extra::extract::cookie::{Cookie, SameSite};
use axum_extra::extract::CookieJar;
use axum_extra::headers::UserAgent;
use axum_extra::TypedHeader;
use reqwest::Url;
use serde::Deserialize;
use time::{Duration, SignedDuration};
use crate::AppState;
use crate::entity::ClientModel;

const SESSION_COOKIE: &str = "session";


#[derive(Template)]
#[template(path = "login.html")]
pub struct LoginPage{
    pub failed: bool,
    pub sessions: Vec<ClientModel>
}

#[derive(Deserialize)]
pub struct QueryLogin {
    pub target: Option<String>
}

pub async fn login(
    State(state): State<AppState>,
    Query(params_login): Query<QueryLogin>,
    jar: CookieJar,
) -> Response {
    if check_auth(jar, state.clone()).await {
        // already logged in
        Redirect::to(&params_login.target.unwrap_or("/".to_owned())).into_response()

    }
    else {
        let clients = state.db.get_clients().await.unwrap();
        let p = LoginPage {
            failed: false,
            sessions: clients.sessions,
        };
        Html(p.render().unwrap()).into_response()
    }
}

#[derive(Deserialize)]
pub struct FormLogin {
    password: String,
    session_id: u32,
}

pub async fn post_login(
    State(state): State<AppState>,
    Query(params_login): Query<QueryLogin>,
    user_agent: Option<TypedHeader<UserAgent>>,
    Form(form_login): Form<FormLogin>,
) -> Response {
    let session_id = match form_login.session_id {
        0 => None,
        id => Some(id),
    };
    let user_agent = user_agent.map(|x| x.to_string());
    let result = state.db.login(session_id, form_login.password, user_agent).await.unwrap();
    if let Some(session_token) = result {
        let target = match params_login.target {
            Some(target) if is_safe_target(&target) => target,
            _ => "/".into()
        };
        let r = Redirect::to(&target);
        let jar = CookieJar::new();
        let cookie = Cookie::build((SESSION_COOKIE, session_token))
            .http_only(true)
            .secure(true) // need to drop if anyone uses this without ssl rev proxy
            .same_site(SameSite::Strict)
            .path("/")
            // client side doesnt need to invalidate session token
            .max_age(SignedDuration::weeks(50000));

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





pub async fn require_web_auth(
    State(state): State<AppState>,
    jar: CookieJar,
    request: Request,
    next: Next,
) -> Response {

    if check_auth(jar, state).await {
        // call route handler if logged in
        next.run(request).await
    }
    else {
        // if not, generate target query
        let uri = request
            .extensions()
            .get::<OriginalUri>()
            .map(|OriginalUri(u)| u)
            .unwrap_or_else(|| request.uri());
        let target = uri.path_and_query().map(|pq| pq.as_str()).unwrap_or("/");
        let query = form_urlencoded::Serializer::new(String::new())
            .append_pair("target", target)
            .finish();

        // redirect
        Redirect::to(&format!("/login?{}", query)).into_response()
    }
}

async fn check_auth(jar: CookieJar, state: AppState) -> bool {
    let session_cookie = jar.get(SESSION_COOKIE);
    if let Some(session_cookie) = session_cookie {
        let session_token = session_cookie.value().to_string();
        state.db.authenticate_session(session_token).await.unwrap()
    }
    else {
        false
    }
}