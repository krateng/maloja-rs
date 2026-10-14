use serde::Serialize;
use strum_macros::{Display, EnumString};
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema, Clone, EnumString, Display)]
pub enum MalojaError {
    /// Artist with that ID does not exist in Database
    ArtistNotFound { id: u32 },
    /// Track with that ID does not exist in Database
    TrackNotFound { id: u32 },
    /// Album with that ID does not exist in Database
    AlbumNotFound { id: u32 },
    /// A field needed to complete the entity submission was not provided
    MissingField { field: String },
    /// Could not connect to the Database
    DatabaseConnectionError { message: String },
    /// Error in a Database Operation
    DatabaseError { message: String },
    /// Error while interacting with the filesystem
    FilesystemError { message: String },
    /// User input could not be read into a valid internal object
    ParseError { message: String },
    /// Generic Server error - likely bug!
    InternalError { message: String },
    /// Time-related error that was not caused by incorrect user input
    TimeError { message: String },
    /// Query string does not resolve to a valid query
    QueryError { message: String },
}

impl std::error::Error for MalojaError {

}

impl From<sea_orm::DbErr> for MalojaError {
    fn from(e: sea_orm::DbErr) -> Self {
        MalojaError::DatabaseError { message: e.to_string() }
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
