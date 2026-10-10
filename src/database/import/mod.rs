pub mod maloja;

use crate::configuration::FOLDERS;
use maloja::import_maloja;
use std::{fs, io};

pub async fn import() -> Result<(i32, i32), io::Error> {
    let import_folder = FOLDERS.data.join("import");

    let (mut imported, mut failed): (i32, i32) = (0, 0);
    if import_folder.exists() {
        for entry in fs::read_dir(import_folder)? {
            let entry = entry?;
            let _result = match entry.file_name().to_str() {
                Some("maloja_export.json") => import_maloja(entry.path()).await,
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
