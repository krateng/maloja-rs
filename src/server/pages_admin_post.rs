use askama::Template;
use axum::extract::{Query, State};
use axum::Form;
use axum::http::header::{COOKIE, WWW_AUTHENTICATE};
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use axum::response::{Html, IntoResponse, Response, Redirect};
use axum_extra::extract::cookie::{Cookie, SameSite};
use axum_extra::extract::CookieJar;
use axum_extra::{TypedHeader, headers::UserAgent};
use serde::Deserialize;
use serde_with::{serde_as, DisplayFromStr};
use time::Duration;
use crate::AppState;
use crate::entity::ClientType;
use super::pages_admin::{QueryLogin, LoginPage, admin_auth};

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
    state.db.login(session_id, form_login.password, user_agent).await.unwrap();
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

#[serde_as]
#[derive(Deserialize, Debug)]
#[serde(tag = "action", rename_all = "snake_case", deny_unknown_fields)]
pub enum FormAuth {
    Revoke {
        #[serde_as(as = "DisplayFromStr")]
        client_id: u32
    },
    Reissue {
        #[serde_as(as = "DisplayFromStr")]
        client_id: u32
    },
    ChangeType {
        #[serde_as(as = "DisplayFromStr")]
        client_id: u32,
        target_type: ClientType
    },
    Create {
        client_type: ClientType,
        identifier: String,
        //expire: String,
    },
    SetPassword {
        password: String,
        password_confirm: String,
    }
}

pub async fn post_admin_auth(
    State(state): State<AppState>,
    Form(form_auth): Form<FormAuth>,
) -> Response {
    match form_auth {
        FormAuth::ChangeType { client_id, target_type } => {
            state.db.change_client_type(client_id, target_type).await.unwrap();
        }
        FormAuth::Reissue { client_id } => {
            let token_display = state.db.rotate_key(client_id, Some(true)).await.unwrap();
        }
        FormAuth::Revoke { client_id } => {
            // also rotate key, new one is ignored but just in case
            state.db.rotate_key(client_id, Some(false)).await.unwrap();
        }
        FormAuth::Create { client_type, identifier } => {
            let token_display = state.db.create_api_key(identifier).await.unwrap();
        }
        _ => todo!()
    }
    redirect_with_flash_cookie("/admin_auth", "Success or smth")
}

fn redirect_with_flash_cookie(path: &str, msg: &str) -> Response {
    let jar = CookieJar::new();
    let cookie = Cookie::build(("flash_message", msg.to_owned()))
        .http_only(true)
        .same_site(SameSite::Strict)
        .path(path.to_owned())
        .max_age(Duration::seconds(10))
        .build();
    (jar.add(cookie), Redirect::to(path)).into_response()
}
