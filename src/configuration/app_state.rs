use std::{env, fs};
use std::fs::{remove_file, OpenOptions};
use std::path::PathBuf;
use confique::Config;
use log::warn;
use sea_orm::DatabaseConnection;
use crate::configuration::logging::{display_envvar, display_path};
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

fn ensure_all_folders(folders: &ApplicationFolders) -> Result<(), MalojaError> {
    let mut success: bool = true;

    let folder_logic: [(&PathBuf, &str, &str, bool); 3] = [
        (&folders.data, "MALOJA_DATA_PATH", "/var/lib/maloja", true),
        (&folders.config, "MALOJA_CONFIG_PATH", "/etc/maloja", false),
        (&folders.logs, "MALOJA_LOG_PATH", "/var/log/maloja", true),
    ];

    for (folder, envvar, _default, writable) in folder_logic.iter() {
        match fs::create_dir_all(folder) {
            Ok(_) => {
                if *writable {
                    let test_file = folder.join(".write_test");
                    let result = OpenOptions::new().write(true).create(true).open(&test_file);
                    match result {
                        Ok(_) => {
                            let _ = remove_file(test_file);
                        }
                        Err(e) => {
                            println!(
                                "No write access to {}: {}. Make sure to set the environment variable {} to a writable directory.",
                                display_path(folder),
                                e,
                                display_envvar(envvar)
                            );
                            success = false;
                        }
                    }
                }
            }
            Err(e) => {
                // logging isn't setup yet
                println!(
                    "Failed to create {}: {}. Make sure to set the environment variable {} to a writable directory.",
                    display_path(folder),
                    e,
                    display_envvar(envvar)
                );
                success = false;
            }
        }
    }
    if !success {
        panic!("Failed to initialize application folders");
    }
    Ok(())

    // TODO makes this all errors
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
        ensure_all_folders(&af)?;
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

