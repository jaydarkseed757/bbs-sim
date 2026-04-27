use serde::Deserialize;

use crate::bbs::data::Board;

// kept for tests / future use
#[allow(dead_code)]
pub struct BoardStore {
    pub boards: Vec<Board>,
}

#[allow(dead_code)]
impl BoardStore {
    pub fn new() -> Self { Self { boards: vec![] } }
    pub fn get_board(&self, id: &str) -> Option<&Board> {
        self.boards.iter().find(|b| b.id == id)
    }
}

// ---------------------------------------------------------------------------
// TOML loader
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct BbsToml {
    #[serde(default)]
    boards: Vec<Board>,
}

/// Load boards for a BBS from `data/bbs/<slug>.toml`.
/// Returns an empty Vec if the file is missing or malformed.
pub fn load_boards(slug: &str) -> Vec<Board> {
    if slug.is_empty() { return vec![]; }
    let path = format!("data/bbs/{}.toml", slug);
    let contents = match std::fs::read_to_string(&path) {
        Ok(s)  => s,
        Err(_) => return vec![],
    };
    match toml::from_str::<BbsToml>(&contents) {
        Ok(data) => data.boards,
        Err(e)   => { eprintln!("parse error in {}: {}", path, e); vec![] }
    }
}

// kept so existing call-sites that reference the symbol still compile
#[allow(dead_code)]
pub fn hardcoded_boards() -> Vec<Board> { vec![] }
