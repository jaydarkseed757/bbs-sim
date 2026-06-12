use serde::Deserialize;

use crate::bbs::data::Caller;

#[derive(Deserialize)]
struct CallersToml {
    #[serde(default)]
    callers: Vec<Caller>,
}

pub fn load_callers(slug: &str) -> Vec<Caller> {
    let path = format!("data/callers/{}.toml", slug);
    let text = match std::fs::read_to_string(&path) {
        Ok(t) => t,
        Err(_) => return vec![],
    };
    match toml::from_str::<CallersToml>(&text) {
        Ok(c) => c.callers,
        Err(_) => vec![],
    }
}
