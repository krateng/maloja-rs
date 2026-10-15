use crate::{database, setup};
use std::fs;
use crate::configuration::CONFIG;

mod api;
mod fixtures;
mod parsing;
mod import;

pub(crate) async fn common() {

    fs::remove_dir_all("./testing/data")
        .or_else(|e| if e.kind() == std::io::ErrorKind::NotFound { Ok(()) } else { Err(e) })
        .expect("Could not remove ./testing/data");
    fs::remove_dir_all("./testing/config")
        .or_else(|e| if e.kind() == std::io::ErrorKind::NotFound { Ok(()) } else { Err(e) })
        .expect("Could not remove ./testing/config");
    fs::remove_dir_all("./testing/log")
        .or_else(|e| if e.kind() == std::io::ErrorKind::NotFound { Ok(()) } else { Err(e) })
        .expect("Could not remove ./testing/log");

    setup(true).await;
}
