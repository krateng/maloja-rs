use crate::{database, setup};
use std::fs;
use crate::configuration::CONFIG;
use rstest::{fixture, rstest};
use tempfile::TempDir;

mod api;
mod fixtures;
mod parsing;
mod import;





pub(crate) async fn common() {

    setup(true).await.unwrap();
}
