use crate::database;
use std::fs;

mod api;
mod fixtures;
mod parsing;

pub(crate) async fn common() {
    dotenvy::from_filename(".env.test").ok();

    fs::remove_dir_all("./testing/data")
        .or_else(|e| if e.kind() == std::io::ErrorKind::NotFound { Ok(()) } else { Err(e) })
        .expect("Could not remove ./testing/data");
    fs::remove_dir_all("./testing/config")
        .or_else(|e| if e.kind() == std::io::ErrorKind::NotFound { Ok(()) } else { Err(e) })
        .expect("Could not remove ./testing/config");
    fs::remove_dir_all("./testing/log")
        .or_else(|e| if e.kind() == std::io::ErrorKind::NotFound { Ok(()) } else { Err(e) })
        .expect("Could not remove ./testing/log");

    database::init_db().await.unwrap();
}
