use tempfile::TempDir;
use crate::{database, setup, AppState};
use crate::configuration::app_state::ApplicationFolders;

mod api;
mod fixtures;
mod parsing;
mod import;



fn create_sandbox() -> TempDir {
    tempfile::Builder::new()
        .prefix("test-maloja-")
        .tempdir_in("/dev/shm")
        .unwrap()
}

pub(crate) async fn common() -> AppState {
    // create ram directories so we dont have constant disk access
    let folders = ApplicationFolders {
        data: create_sandbox().path().to_owned(),
        config: create_sandbox().path().to_owned(),
        logs: create_sandbox().path().to_owned(),
    };

    let state = AppState::new(Some(folders)).await.unwrap();

    setup(true, &state).await.unwrap();

    state
}
