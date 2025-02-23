use axum::extract::Path;
use axum::response::{Html, IntoResponse, Response};
//use dynja::Template;
use askama::Template;
use crate::database;
use crate::entity::album::AlbumRead;
use crate::entity::artist::ArtistRead;
use crate::entity::track::TrackRead;
use crate::uri::PathEntity;

#[derive(Template)]
#[template(path = "info_artist.html")]
struct ArtistPage {
    artist: ArtistRead,
}
pub async fn info_artist(Path(params_path): Path<PathEntity>) -> Response {
    let result = database::repository::artist_info(params_path.id).await.unwrap();
    let p = ArtistPage {
        artist: result,
    };
    Html(p.render().unwrap()).into_response()
}

#[derive(Template)]
#[template(path = "info_track.html")]
struct TrackPage {
    track: TrackRead,
}
pub async fn info_track(Path(params_path): Path<PathEntity>) -> Response {
    let result = database::repository::track_info(params_path.id).await.unwrap();
    let p = TrackPage {
        track: result,
    };
    Html(p.render().unwrap()).into_response()
}

#[derive(Template)]
#[template(path = "info_album.html")]
struct AlbumPage {
    album: AlbumRead,
}
pub async fn info_album(Path(params_path): Path<PathEntity>) -> Response {
    let result = database::repository::album_info(params_path.id).await.unwrap();
    let p = AlbumPage {
        album: result,
    };
    Html(p.render().unwrap()).into_response()
}



#[derive(Template)]
#[template(path = "about.html")]
struct AboutPage {
    version: String,
}
pub async fn about() -> Response {
    let p = AboutPage {
        version: "2.5.4".to_string(),
    };

    Html(p.render().unwrap()).into_response()
}
