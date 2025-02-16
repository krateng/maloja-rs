mod fixtures;

use super::*;
use std::fs;

async fn common() {
    std::env::set_var("MALOJA_DATA_PATH", "./testing/data");
    std::env::set_var("MALOJA_CONFIG_PATH", "./testing/config");
    std::env::set_var("MALOJA_LOG_PATH", "./testing/log");

    fs::remove_dir_all("./testing/data");
    fs::remove_dir_all("./testing/config");
    fs::remove_dir_all("./testing/log");

    database::init_db().await.unwrap();

    fixtures::fixture().await;
}


//#[tokio::test]
//async fn run_server() {
//    server::run_server().await;
//}