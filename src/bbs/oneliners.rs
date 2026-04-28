use serde::Deserialize;

use crate::bbs::data::Oneliner;

#[derive(Deserialize)]
struct OnelinerToml {
    #[serde(default)]
    lines: Vec<Oneliner>,
}

pub fn load_oneliners(slug: &str) -> Vec<Oneliner> {
    let path = format!("data/oneliners/{}.toml", slug);
    let text = match std::fs::read_to_string(&path) {
        Ok(t) => t,
        Err(_) => return vec![],
    };
    match toml::from_str::<OnelinerToml>(&text) {
        Ok(f) => f.lines,
        Err(_) => vec![],
    }
}
