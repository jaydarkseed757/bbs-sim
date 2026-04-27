use serde::Deserialize;

use crate::bbs::data::FileSection;

#[derive(Deserialize)]
struct FilesToml {
    #[serde(default)]
    sections: Vec<FileSection>,
}

pub fn load_files(slug: &str) -> Vec<FileSection> {
    let path = format!("data/files/{}.toml", slug);
    let text = match std::fs::read_to_string(&path) {
        Ok(t) => t,
        Err(_) => return vec![],
    };
    match toml::from_str::<FilesToml>(&text) {
        Ok(f) => f.sections,
        Err(_) => vec![],
    }
}
