use itertools::{any, concat, Itertools};
use regex::Regex;
use crate::entity::{ArtistWrite, ArtistWriteContext, TrackWrite};

#[derive(Debug, Clone)]
pub struct MalojaParsingRuleset {
    name: String,
    author: String,
    author_email: String,
    rules: Vec<MalojaParsingRule>,
}

#[derive(Debug, Clone)]
pub enum MalojaParsingRule {
    ArtistRegex {
        capture_name: Regex,
        write_artists: Vec<String>,
    },
    AlbumRegex {
        input: Regex,
    },
    TrackRegex {
        capture_title: Regex,
        match_artists_any: Regex,
        match_artists_all: Regex,
        write_title: String,
        add_artists: Vec<String>,
    },
    IgnoreArtist {
        regex: Regex,
    },
    ArtistAsAlias {
        artist_match: Regex,
        real_name: String,
    }
}

// TODO make these static
fn hard_coded_rules() -> Vec<MalojaParsingRule> {
    vec![
        // featuring in title
        MalojaParsingRule::TrackRegex {
            capture_title: r"(.+) (?:feat\.?|ft\.?|featuring) (.+)".parse().unwrap(),
            match_artists_any: ".*".parse().unwrap(),
            match_artists_all: ".*".parse().unwrap(),
            write_title: "$1".to_string(),
            add_artists: vec!["$2".to_string()],
        },
        // featuring in title with parenthesis
        MalojaParsingRule::TrackRegex {
            capture_title: r"(.+) \((?:feat\.?|ft\.?|featuring) (.+)\)".parse().unwrap(),
            match_artists_any: ".*".parse().unwrap(),
            match_artists_all: ".*".parse().unwrap(),
            write_title: "$1".to_string(),
            add_artists: vec!["$2".to_string()],
        },
        // featuring in artist
        MalojaParsingRule::ArtistRegex {
            capture_name: r"(.+) (?:feat\.?|ft\.?|featuring) (.+)".parse().unwrap(),
            write_artists: vec!["$1".to_string(), "$2".to_string()],
        },
        // featuring in artist with parenthesis
        MalojaParsingRule::ArtistRegex {
            capture_name: r"(.+) \((?:feat\.?|ft\.?|featuring) (.+)\)".parse().unwrap(),
            write_artists: vec!["$1".to_string(), "$2".to_string()],
        },
        // ; artist separator
        MalojaParsingRule::ArtistRegex {
            capture_name: r"(.+);(.+)".parse().unwrap(),
            write_artists: vec!["$1".to_string(), "$2".to_string()],
        },
    ]
}

impl MalojaParsingRule {
    // returns bool whether further rule application should be stopped (final result)
    fn apply_to_track(&self, write: TrackWrite) -> (TrackWrite, bool) {
        // no need to do it on id-identified or invalid writes
        if let (Some(title), Some(artists)) = (&write.title, &write.artists) {
            match &self {
                MalojaParsingRule::TrackRegex { capture_title, match_artists_any, match_artists_all, write_title, add_artists } => {
                    if
                    artists.iter().any(|a| { if let Some(name) = &a.name { match_artists_any.is_match(&*name) } else { false } })
                        && artists.iter().all(|a| { if let Some(name) = &a.name { match_artists_all.is_match(&*name) } else { false } })
                        // we need the extra if check because otherwise our replace will happily take the title with no match and add it as a new artist
                        && capture_title.is_match(title)
                    {
                        let mut write = write.clone();
                        write.title = Some(capture_title.replace(title, write_title).to_string());
                        let new_artists: Vec<String> = add_artists.iter().map(|a| capture_title.replace(title, a).to_string()).collect();
                        let new_artist_writes: Vec<ArtistWriteContext> = new_artists.into_iter().map(|a| ArtistWriteContext {
                            id: None,
                            name: Some(a),
                            alias: None,
                            primary: false,
                            mbid: None,
                            spotify_id: None,
                        }).collect();
                        write.artists = Some(concat(vec![artists.clone(), new_artist_writes]));
                        (write, true)

                    }
                    else {
                        (write, true)
                    }
                },
                MalojaParsingRule::IgnoreArtist { regex } => todo!(),
                _ => (write, true),
            }
        } else {
            (write, true)
        }
    }


    // returns bool whether further rule application should be stopped (final result)
    fn apply_to_artist(&self, write: ArtistWriteContext) -> (Vec<ArtistWriteContext>, bool) {
        if let Some(name) = &write.name {
            match &self {
                MalojaParsingRule::ArtistRegex { capture_name, write_artists } => {
                    if let Some(name) = &write.name && capture_name.is_match(name) {
                        let artists = write_artists.iter().map(|a| capture_name.replace(name, a).to_string());
                        let artist_writes: Vec<ArtistWriteContext> = artists.map(|a| ArtistWriteContext {
                            id: None,
                            name: Some(a),
                            alias: None,
                            primary: write.primary,
                            mbid: None,
                            spotify_id: None,
                        }).collect();
                        (artist_writes, true)
                    }
                    else {
                        (vec![write], true)
                    }

                },
                MalojaParsingRule::ArtistAsAlias { artist_match, real_name } => {
                    if let Some(name) = &write.name && artist_match.is_match(name) {
                        (vec![ArtistWriteContext {
                            id: write.id,
                            name: Some(real_name.clone()),
                            alias: write.name,
                            primary: write.primary,
                            mbid: write.mbid,
                            spotify_id: write.spotify_id,
                        }], false)
                    }
                    else {
                        (vec![write], true)
                    }

                },
                _ => (vec![write], true)
            }
        } else {
            (vec![write], true)
        }
    }
}

pub fn apply_artist_rules(mut writes: Vec<ArtistWriteContext>) -> Vec<ArtistWriteContext> {
    let rules = hard_coded_rules();
    let mut done_writes: Vec<ArtistWriteContext> = vec![];
    for rule in rules {
        let mut remaining_writes: Vec<ArtistWriteContext> = vec![];
        for write in writes {
            let (new_writes, cont) = rule.apply_to_artist(write);
            if cont {
                remaining_writes.extend(new_writes);
            }
            else {
                done_writes.extend(new_writes);
            }
        }
        writes = remaining_writes;
    }
    writes.extend(done_writes);
    writes
}

pub fn apply_track_rules(mut write: TrackWrite) -> TrackWrite {
    write.artists = write.artists.map(apply_artist_rules);

    let rules = hard_coded_rules();
    let mut cont = true;
    for rule in rules {
        if cont {
            (write, cont) = rule.apply_to_track(write);

        }
    }
    write
}

fn debug(write: &TrackWrite) -> String {
    format!("{} - {}", write.artists.clone().unwrap().into_iter().map(|x| x.name.unwrap()).join(", "), write.title.clone().unwrap())
}