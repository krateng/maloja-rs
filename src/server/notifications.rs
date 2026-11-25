// ok we're gonna try to build a fully js-free notification system and later use js as an optional override

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use sea_query::Iden;
use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize)]
pub enum Notification {
    ApiKeyGenerated { token: String },
    ApiKeyReissued { token: String },
    PasswordChanged,
    // Errors
    PasswordInvalid,
    PasswordsDoNotMatch,
    WrongPassword,
}

pub enum ToastType {
    Success,
    Info,
    Warning,
    Error,
}

impl Notification {
    pub fn into_toast(self) -> Toast {
        let level = match self {
            Self::ApiKeyGenerated { token: _ } |
            Self::ApiKeyReissued { token: _ } |
            Self::PasswordChanged => ToastType::Success,

            Self::WrongPassword |
            Self::PasswordInvalid |
            Self::PasswordsDoNotMatch => ToastType::Error,
        };
        let title = match self {
            Self::ApiKeyGenerated { token: _ } |
            Self::ApiKeyReissued { token: _ } => "Key generated".to_string(),
            Self::PasswordChanged => "Password changed".to_string(),
            Self::PasswordInvalid |
            Self::PasswordsDoNotMatch => "Password not changed".to_string(),
            Self::WrongPassword => "Wrong password".to_string(),
        };
        let text = match self {
            Self::ApiKeyGenerated { token: _ } |
            Self::ApiKeyReissued { token: _ } => Some("Your key:".to_string()),
            Self::PasswordInvalid => Some("Password needs to be at least 12 characters".to_string()),
            Self::PasswordsDoNotMatch => Some("Passwords do not match".to_string()),

            _ => None,
        };
        let copy_text = match self {
            Self::ApiKeyGenerated { token } | Self::ApiKeyReissued { token } => Some(token),
            _ => None,
        };

        Toast {
            level,
            title,
            text,
            copy_text,
        }
    }

    pub fn to_cookie(&self) -> String {
        let json = serde_json::to_vec(self).expect("Notice always serializes");
        URL_SAFE_NO_PAD.encode(json)
    }
    pub fn from_cookie(cookie: &str) -> Option<Self> {
        let bytes = URL_SAFE_NO_PAD.decode(cookie).ok()?;
        serde_json::from_slice(&bytes).ok()
    }
}



pub struct Toast {
    pub title: String,
    pub text: Option<String>,
    pub copy_text: Option<String>,
    pub level: ToastType,
}

impl Toast {
    pub fn to_css_class(&self) -> String {
        match self.level {
            ToastType::Success => "success".to_string(),
            ToastType::Info => "info".to_string(),
            ToastType::Warning => "warning".to_string(),
            ToastType::Error => "error".to_string(),
        }
    }
}
