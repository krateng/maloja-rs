use itertools::{any, concat, Itertools};
use regex::Regex;
use sea_query::Iden;
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
    fn apply_to_track(&self, write: TrackWrite) -> TrackWrite {
        // no need to do it on id-identified or invalid writes
        if let (Some(title), Some(artists)) = (&write.title, &write.artists) {
            let mut write = write.clone();
            match &self {
                MalojaParsingRule::TrackRegex { capture_title, match_artists_any, match_artists_all, write_title, add_artists } => {
                    if
                    artists.iter().any(|a| { if let Some(name) = &a.name { match_artists_any.is_match(&*name) } else { false } })
                        && artists.iter().all(|a| { if let Some(name) = &a.name { match_artists_all.is_match(&*name) } else { false } })
                    {
                        // we need the extra if check because otherwise our replace will happily take the title with no match and add it as a new artist
                        if capture_title.is_match(title) {
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
                        }

                    }
                },
                MalojaParsingRule::IgnoreArtist { regex } => todo!(),
                _ => {},
            }
            write
        } else {
            write
        }
    }

    fn apply_to_artist(&self, write: ArtistWriteContext) -> Vec<ArtistWriteContext> {
        if let Some(name) = &write.name {
            return match &self {
                MalojaParsingRule::ArtistRegex { capture_name, write_artists } => {
                    let artists = write_artists.iter().map(|a| capture_name.replace(name, a).to_string());
                    let artist_writes: Vec<ArtistWriteContext> = artists.map(|a| ArtistWriteContext {
                        id: None,
                        name: Some(a),
                        alias: None,
                        primary: write.primary,
                        mbid: None,
                        spotify_id: None,
                    }).collect();
                    artist_writes
                }
                _ => vec![write]
            }
        }
        vec![write]
    }
}

pub fn apply_artist_rules(mut writes: Vec<ArtistWriteContext>) -> Vec<ArtistWriteContext> {
    let rules = hard_coded_rules();
    for rule in rules {
        // each time get all existing, apply the rule, get maybe more back
        // TODO overthink this sequence
        let mut new_writes = vec![];
        for write in writes {
            new_writes.append(rule.apply_to_artist(write).as_mut());
        }
        writes = new_writes;
    }
    writes
}

pub fn apply_track_rules(mut write: TrackWrite) -> TrackWrite {
    let rules = hard_coded_rules();
    //println!("Before: {}", debug(&write));
    for rule in rules {
        write = rule.apply_to_track(write);
        write.artists = write.artists.map(|a| apply_artist_rules(a));
        //println!("After rule: {:?}", debug(&write));
    }
    //println!("Done!");
    write
}

fn debug(write: &TrackWrite) -> String {
    format!("{} - {}", write.artists.clone().unwrap().into_iter().map(|x| x.name.unwrap()).join(", "), write.title.clone().unwrap())
}