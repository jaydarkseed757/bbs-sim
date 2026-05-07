pub fn load_banner(slug: &str) -> Option<String> {
    let path = format!("data/banners/{}.ans", slug);
    std::fs::read_to_string(&path).ok()
}
