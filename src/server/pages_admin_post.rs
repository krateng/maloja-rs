use axum::extract::{Query, State};
use axum::Form;
use axum::response::{Html, IntoResponse, Response, Redirect};
use axum_extra::extract::cookie::{Cookie, SameSite};
use axum_extra::extract::CookieJar;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};
use time::Duration;
use crate::AppState;
use crate::entity::ClientType;
use crate::server::notifications::Notification;

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
            Redirect::to("/admin_auth").into_response()
        }
        FormAuth::Reissue { client_id } => {
            let token_display = state.db.rotate_key(client_id, Some(true)).await.unwrap();
            redirect_with_flash_cookie("/admin_auth", Notification::ApiKeyReissued { token: token_display })
        }
        FormAuth::Revoke { client_id } => {
            // also rotate key, new one is ignored but just in case
            state.db.rotate_key(client_id, Some(false)).await.unwrap();
            Redirect::to("/admin_auth").into_response()
        }
        FormAuth::Create { client_type, identifier } => {
            let token_display = state.db.create_api_key(identifier).await.unwrap();
            redirect_with_flash_cookie("/admin_auth", Notification::ApiKeyGenerated { token: token_display })
        }
        FormAuth::ClearUnused { client_type } => {
            state.db.clear_unused(client_type).await.unwrap();
            Redirect::to("/admin_auth").into_response()
        }
        FormAuth::SetPassword { password, password_confirm } => {
            if password != password_confirm {
                redirect_with_flash_cookie("/admin_auth", Notification::PasswordsDoNotMatch)
            }
            else {
                match state.db.set_password(password).await {
                    Ok(_) => redirect_with_flash_cookie("/admin_auth", Notification::PasswordChanged),
                    Err(_) => redirect_with_flash_cookie("/admin_auth", Notification::PasswordInvalid)
                }
            }
        }
    }
}

pub fn redirect_with_flash_cookie(path: &str, msg: Notification) -> Response {
    let jar = CookieJar::new();
    let cookie = Cookie::build(("flash_message", msg.to_cookie()))
        .http_only(true)
        .same_site(SameSite::Strict)
        .path(path.to_owned())
        .max_age(Duration::seconds(10))
        .build();
    (jar.add(cookie), Redirect::to(path)).into_response()
}
