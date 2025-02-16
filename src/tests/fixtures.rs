use rand::prelude::SliceRandom;
use rand::{thread_rng, Rng};
use crate::database;
use crate::entity::artist::ArtistWrite;
use crate::entity::scrobble::ScrobbleWrite;
use crate::entity::track::TrackWrite;

const ARTIST_PRENAMES_FEMALE: &[&str] = &[
    "Jennie", "Soyeon", "Lalisa", "Dahyun", "Mina", "Tzuyu", "Jihyo", "Yura", "Nancy", "Yuqi", "Seolhyun",
    "Frieren", "Zelda", "Aqua", "Chris", "Azula", "Rem",
    "Sansa", "Gilly", "Melisandre", "Doreah", "Nymeria",
    "Gwen", "Livia", "Evennia", "Nika", "Devona", "Kehanni", "Jamei",
];
const ARTIST_PRENAMES_MALE: &[&str] = &[
    "Denethor", "Ecthelion", "Finarfin", "Fingolfin",
    "Celeborn", "Celebrimbor", "Elrond", "Elrohir", "Elladan",
    "Timo", "Ricardo", "Marcus", "Fernando", "Marco", "Thomas", "Pável", "Alexander",
    "Matthieu", "Ludovic", "Christian", "Antônio", "Sami", "Mario", "Serdar"
];

const ARTIST_PARTICLES_SHARED: &[&str] = &[
    "de ", "del ", "d'", "von ", "van ",
    "Mc", "Mac", "Al-",
];
const ARTIST_PARTICLES_FEMALE: &[&str] = &[
    "Ferch ", "Nic ", "Bint ", "Bat-",
];
const ARTIST_PARTICLES_MALE: &[&str] = &[
    "Ap ", "Fitz", "Ibn ", "Ben-",
];
const ARTIST_SURNAMES: &[&str] = &[
    "Schupfnudel", "Rahmschnitzel", "Maultasche", "Knoepfli",
    "Gipfeli", "Laugenstange", "Reisauflauf", "Spiegelei",
];

const SONG_DETERMINERS: &[&str] = &[
    "A", "The", "This", "That", "Some", "My", "Your", "Our",
];
const SONG_NOUNS: &[&str] = &[
    "Game", "Clash", "Storm", "Feast", "Dance", "Dream",
];
const SONG_PREPOSITIONS: &[&str] = &[
    "of", "at", "on", "in", "by",
    "into", "onto", "from", "with",
];
const SONG_NOUNS_PLURAL: &[&str] = &[
    "Legends", "Shadows", "Echoes", "Whispers",
    "Fires", "Waves", "Hearts", "Tales", "Stars",
    "Dreams", "Spirits", "Secrets", "Storms",
];


pub async fn fixture() {

    const ARTISTS_AMOUNT: u32 = 7;
    const TRACKS_AMOUNT: u32 = 20;
    const SCROBBLES_AMOUNT: u32 = 40;

    

    let mut artists = vec![];
    for _i in (0..ARTISTS_AMOUNT) {
        let (artist_prenames, artist_particles) = match thread_rng().gen_bool(0.5) {
            true => { (ARTIST_PRENAMES_FEMALE, [ARTIST_PARTICLES_FEMALE, ARTIST_PARTICLES_SHARED].concat()) },
            false => { (ARTIST_PRENAMES_MALE, [ARTIST_PARTICLES_MALE, ARTIST_PARTICLES_SHARED].concat()) }
        };
        artists.push(ArtistWrite {
            id: None,
            name: Some(format!("{} {}{}",
                               artist_prenames.choose(&mut thread_rng()).unwrap(),
                               artist_particles.choose(&mut thread_rng()).unwrap(),
                               ARTIST_SURNAMES.choose(&mut thread_rng()).unwrap()
            )),
            mbid: None,
            spotify_id: None,
        });
    }
    
    let mut tracks = vec![];
    for _i in (0..TRACKS_AMOUNT) {

        let artist_amount = *[1usize, 1usize, 1usize, 2usize, 2usize, 3usize].choose(&mut thread_rng()).unwrap();
        let track_artists: Vec<&ArtistWrite> = artists.choose_multiple(&mut thread_rng(), artist_amount).collect();
        let track_artists = track_artists.into_iter().map(|x| x.to_owned()).collect();
        // wtf is this code

        tracks.push(TrackWrite {
            id: None,
            title: Some(format!("{} {} {} {}",
                                SONG_DETERMINERS.choose(&mut thread_rng()).unwrap(),
                                SONG_NOUNS.choose(&mut thread_rng()).unwrap(),
                                SONG_PREPOSITIONS.choose(&mut thread_rng()).unwrap(),
                                SONG_NOUNS_PLURAL.choose(&mut thread_rng()).unwrap()
            )),
            primary_artists: Some(track_artists),
            secondary_artists: None,
            track_length: None,
            album: None,
            mbid: None,
            spotify_id: None,
        });
    }
    let mut time = chrono::Utc::now() - chrono::Duration::days(1);
    let mut timestamp = time.timestamp();

    let mut scrobbles = vec![];
    for _i in (0..SCROBBLES_AMOUNT) {
        scrobbles.push(ScrobbleWrite {
            timestamp: timestamp,
            track: tracks.choose(&mut thread_rng()).unwrap().to_owned(),
            origin: None,
            listen_duration: None,
        });

        timestamp += thread_rng().gen_range(200..2000);
    }

    database::repository::create_scrobbles(scrobbles, false).await.unwrap();

}