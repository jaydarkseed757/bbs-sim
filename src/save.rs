/// Persistent save data — written to `save.json` after every user mutation
/// (post, mail, graffiti, login) via `App::persist_session`, loaded at
/// startup and applied on top of the hardcoded defaults.
use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::bbs::data::{BbsEntry, Board, MailMessage, Oneliner};

const SAVE_PATH: &str = "save.json";
pub const SCHEMA: u32 = 1;

// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct SaveData {
    /// Bumped whenever the format changes in a breaking way.
    #[serde(default)]
    pub schema: u32,

    /// One entry per phonebook BBS (matched by `name`).
    #[serde(default)]
    pub phonebook: Vec<PhonebookSave>,

    /// Full board/mail/oneliner state per BBS slug, accumulated across sessions.
    #[serde(default)]
    pub bbs: HashMap<String, BbsSave>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PhonebookSave {
    pub name:        String,
    pub call_count:  u32,
    pub last_called: Option<String>,
}

/// Everything that can change during a session on one BBS.
#[derive(Serialize, Deserialize, Default, Clone)]
pub struct BbsSave {
    /// Handle the data was saved under — mail to/from this handle is
    /// remapped if the next session logs in under a different one.
    #[serde(default)]
    pub handle:    String,
    #[serde(default)]
    pub boards:    Vec<Board>,
    #[serde(default)]
    pub mail:      Vec<MailMessage>,
    #[serde(default)]
    pub oneliners: Vec<Oneliner>,
}

// ---------------------------------------------------------------------------

impl SaveData {
    /// Load from `save.json`, returning a default if the file is missing or corrupt.
    pub fn load() -> Self {
        let path = std::path::Path::new(SAVE_PATH);
        if !path.exists() {
            return Self::default();
        }
        let text = match std::fs::read_to_string(path) {
            Ok(s)  => s,
            Err(e) => { eprintln!("[save] read error: {}", e); return Self::default(); }
        };
        match serde_json::from_str::<SaveData>(&text) {
            Ok(mut d) => {
                if d.schema != SCHEMA {
                    eprintln!("[save] schema mismatch (got {}, want {}), ignoring", d.schema, SCHEMA);
                    return Self::default();
                }
                d.schema = SCHEMA;
                d
            }
            Err(e) => {
                eprintln!("[save] corrupt save file, starting fresh: {}", e);
                Self::default()
            }
        }
    }

    /// Write to `save.json`, printing to stderr on failure (never panics).
    pub fn write(&mut self) {
        self.schema = SCHEMA;
        match serde_json::to_string_pretty(self) {
            Ok(json) => {
                if let Err(e) = std::fs::write(SAVE_PATH, &json) {
                    eprintln!("[save] write error: {}", e);
                }
            }
            Err(e) => eprintln!("[save] serialize error: {}", e),
        }
    }

    // ── Helpers called by App ────────────────────────────────────────────────

    /// Apply saved call counts / last-called dates onto the hardcoded phonebook.
    pub fn apply_to_phonebook(&self, mut entries: Vec<BbsEntry>) -> Vec<BbsEntry> {
        for entry in &mut entries {
            if let Some(saved) = self.phonebook.iter().find(|s| s.name == entry.name) {
                entry.call_count  = saved.call_count;
                entry.last_called = saved.last_called.clone();
            }
        }
        entries
    }

    /// Snapshot current phonebook state into the save.
    pub fn capture_phonebook(&mut self, entries: &[BbsEntry]) {
        self.phonebook = entries.iter().map(|e| PhonebookSave {
            name:        e.name.clone(),
            call_count:  e.call_count,
            last_called: e.last_called.clone(),
        }).collect();
    }

    /// Snapshot the current session's mutable BBS data into the save.
    pub fn capture_bbs(
        &mut self,
        slug:      &str,
        handle:    &str,
        boards:    Vec<Board>,
        mail:      Vec<MailMessage>,
        oneliners: Vec<Oneliner>,
    ) {
        if slug.is_empty() { return; }
        self.bbs.insert(slug.to_string(), BbsSave {
            handle: handle.to_string(),
            boards,
            mail,
            oneliners,
        });
    }
}
