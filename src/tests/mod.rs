use std::fs;
use crate::database;

mod fixtures;
mod parsing;
mod api;


pub(crate) async fn common() {
    // TODO MAKE SAFE
    unsafe {
        std::env::set_var("MALOJA_DATA_PATH", "./testing/data");
        std::env::set_var("MALOJA_CONFIG_PATH", "./testing/config");
        std::env::set_var("MALOJA_LOG_PATH", "./testing/log");
    }

    fs::remove_dir_all("./testing/data").expect("Could not remove ./testing/data");
    fs::remove_dir_all("./testing/config").expect("Could not remove ./testing/config");
    fs::remove_dir_all("./testing/log").expect("Could not remove ./testing/log");

    database::init_db().await.unwrap();
}