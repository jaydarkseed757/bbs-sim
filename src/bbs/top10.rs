use crate::bbs::data::TopLists;

pub fn load_top10(slug: &str) -> TopLists {
    let path = format!("data/top10/{}.toml", slug);
    let text = match std::fs::read_to_string(&path) {
        Ok(t) => t,
        Err(_) => return TopLists::default(),
    };
    toml::from_str::<TopLists>(&text).unwrap_or_default()
}
