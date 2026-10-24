pub mod maloja;

use maloja::import_maloja;
use std::{fs, io};
use crate::configuration::app_state::AppState;
use crate::database::errors::MalojaError;

pub async fn import(state: &AppState) -> Result<(i32, i32), MalojaError> {
    let import_folder = state.folders.data.join("import");

    let (mut imported, mut failed): (i32, i32) = (0, 0);
    if import_folder.exists() {
        for entry in fs::read_dir(import_folder)? {
            let entry = entry?;
            match entry.file_name().to_str() {
                Some("maloja_export.json") => import_maloja(entry.path(), state).await?,
                _ => {
                    failed += 1;
                    continue;
                }
            };
            imported += 1;
        }
    }
    Ok((imported, failed))
}
