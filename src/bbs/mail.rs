use serde::Deserialize;

use crate::bbs::data::MailMessage;

#[derive(Deserialize)]
struct MailToml {
    #[serde(default)]
    messages: Vec<MailMessage>,
}

/// Load private mail for `slug` from `data/mail/<slug>.toml`.
/// The `to` field of every message is filled in with `handle`.
pub fn load_mail(slug: &str, handle: &str) -> Vec<MailMessage> {
    if slug.is_empty() { return vec![]; }
    let path = format!("data/mail/{}.toml", slug);
    let contents = match std::fs::read_to_string(&path) {
        Ok(s)  => s,
        Err(_) => return vec![],
    };
    match toml::from_str::<MailToml>(&contents) {
        Ok(mut data) => {
            for msg in &mut data.messages {
                msg.to = handle.to_string();
            }
            data.messages
        }
        Err(e) => { eprintln!("mail parse error in {}: {}", path, e); vec![] }
    }
}
