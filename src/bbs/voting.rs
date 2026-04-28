use serde::Deserialize;

use crate::bbs::data::Poll;

#[derive(Deserialize)]
struct VotingToml {
    #[serde(default)]
    polls: Vec<Poll>,
}

pub fn load_polls(slug: &str) -> Vec<Poll> {
    let path = format!("data/voting/{}.toml", slug);
    let text = match std::fs::read_to_string(&path) {
        Ok(t) => t,
        Err(_) => return vec![],
    };
    match toml::from_str::<VotingToml>(&text) {
        Ok(f) => f.polls,
        Err(_) => vec![],
    }
}
