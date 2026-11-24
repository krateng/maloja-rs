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
    },
    ClearUnused {
        client_type: ClientType,
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
        FormAuth::ClearUnused { client_type } => {
            state.db.clear_unused(client_type).await.unwrap();
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
