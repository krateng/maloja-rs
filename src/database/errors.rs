use axum::http::StatusCode;
use axum::Json;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use strum;
use strum_macros::{Display, EnumString};
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema, Clone, EnumString, Display)]
pub enum MalojaError {
    ArtistNotFound { id: u32 },
    TrackNotFound { id: u32 },
    AlbumNotFound { id: u32 },
    DatabaseConnectionError { message: String },
    DatabaseError { message: String },
    FilesystemError { message: String },
    ParseError { message: String },

}

impl From<sea_orm::DbErr> for MalojaError {
    fn from(e: sea_orm::DbErr) -> Self {
        MalojaError::DatabaseError {
            message: e.to_string(),
        }
    }
}

// TEMPORARY:
impl From<std::io::Error> for MalojaError {
    fn from(e: std::io::Error) -> Self {
        MalojaError::FilesystemError { message: e.to_string() }
    }
}
impl From<serde_json::Error> for MalojaError {
    fn from(e: serde_json::Error) -> Self {
        MalojaError::ParseError { message: e.to_string() }
    }
}


