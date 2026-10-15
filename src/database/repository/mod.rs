pub mod get_or_create;
pub mod history;
pub mod info;
pub mod resolve;
pub mod scrobbles;
pub mod stats;
pub mod models_to_reads;
mod writes_to_models;
mod helpers;

use sea_orm::DatabaseConnection;
pub use models_to_reads::*;
use crate::configuration::app_state::ApplicationFolders;
use crate::database::connect;
use crate::database::errors::MalojaError;

#[derive(Clone)]
pub struct MalojaDB {
    connection: DatabaseConnection,
}

impl MalojaDB {
    pub async fn new(folders: &ApplicationFolders) -> Result<MalojaDB, MalojaError> {
        Ok(Self {
            connection: connect(folders).await?,
        })
    }
}