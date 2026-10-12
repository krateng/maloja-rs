pub mod album;
pub mod album_artist;
pub mod artist;
pub mod scrobble;
pub mod track;
pub mod track_artist;

pub use album::{
    Model as AlbumModel, Entity as Album, AlbumWrite, AlbumRead,
    Column as AlbumColumn, ActiveModel as AlbumActiveModel,
};
pub use artist::{
    Model as ArtistModel, Entity as Artist,
    ArtistWrite, ArtistRead, ArtistWriteContext, ArtistReadContext,
    Column as ArtistColumn, ActiveModel as ArtistActiveModel,
};
pub use track::{
    Model as TrackModel, Entity as Track, TrackWrite, TrackRead,
    Column as TrackColumn, ActiveModel as TrackActiveModel,
};
pub use scrobble::{
    Model as ScrobbleModel, Entity as Scrobble, ScrobbleWrite, ScrobbleRead,
    Column as ScrobbleColumn, ActiveModel as ScrobbleActiveModel,
};
pub use track_artist::{
    Model as TrackArtistModel, Entity as TrackArtist,
    ActiveModel as TrackArtistActiveModel,
};
pub use album_artist::{
    Model as AlbumArtistModel, Entity as AlbumArtist,
    ActiveModel as AlbumArtistActiveModel,
};

use album::Relation as AlbumRelation;
use artist::Relation as ArtistRelation;
use scrobble::Relation as ScrobbleRelation;
use track::Relation as TrackRelation;
use track_artist::Relation as TrackArtistRelation;
use album_artist::Relation as AlbumArtistRelation;
