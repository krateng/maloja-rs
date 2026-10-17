use std::{env, fs};
use std::fs::{exists, remove_file, OpenOptions};
use std::path::PathBuf;
use confique::Config;
use log::warn;
use crate::configuration::config::{get_config_file_path, MalojaConfig};
use crate::database::errors::MalojaError;
use crate::database::repository::MalojaDB;

#[derive(Clone)]
pub struct AppState {
    pub folders: ApplicationFolders,
    pub config: MalojaConfig,
    //pub db_connection: DatabaseConnection,
    pub db: MalojaDB
}

#[derive(Clone)]
pub struct ApplicationFolders {
    pub data: PathBuf,
    pub config: PathBuf,
    pub logs: PathBuf,
}

async fn ensure_folder(folder: &PathBuf, write: bool) -> Result<(), MalojaError> {
    fs::create_dir_all(folder)?;
    if write {
        let test_file = folder.join(".write_test");
        OpenOptions::new().write(true).create(true).truncate(true).open(&test_file)?;
        remove_file(&test_file)?;
    }
    Ok(())
}


impl AppState {
    pub async fn new(custom_folders: Option<ApplicationFolders>) -> Result<Self, MalojaError> {
        let af = custom_folders.unwrap_or(ApplicationFolders {
            // we set the defaults to posix standards, even though maloja is container-oriented first
            // the Containerfile overwrites these
            data: PathBuf::from(
                env::var("MALOJA_DATA_PATH").unwrap_or(String::from("/var/lib/maloja")),
            ),
            config: PathBuf::from(
                env::var("MALOJA_CONFIG_PATH").unwrap_or(String::from("/etc/maloja")),
            ),
            logs: PathBuf::from(env::var("MALOJA_LOG_PATH").unwrap_or(String::from("/var/log/maloja"))),
        });

        ensure_folder(&af.data, true).await?;
        ensure_folder(&af.logs, true).await?;
        ensure_folder(&af.config, false).await?;

        if exists(af.data.join("malojadb.sqlite"))? {
            return Err(MalojaError::FilesystemError {
                message: "Old Maloja installation found. Please use a clean directory and import your data manually.".to_string()
            });
        }

        let mc = MalojaConfig::from_file(get_config_file_path(&af)).unwrap_or_else(|_| {
            // we just load the default values
            warn!("Configuration file loading failed");
            MalojaConfig::builder().load().unwrap()
        });
        let db = MalojaDB::new(&af).await?;
        Ok(AppState {
            folders: af,
            config: mc,
            db: db,
        })

    }
}

