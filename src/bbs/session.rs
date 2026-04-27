#[derive(Debug, Default)]
pub struct Session {
    pub user_handle: Option<String>,
    pub bbs_name: Option<String>,
}

impl Session {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn login(&mut self, handle: String, bbs_name: String) {
        self.user_handle = Some(handle);
        self.bbs_name = Some(bbs_name);
    }

    pub fn logout(&mut self) {
        self.user_handle = None;
        self.bbs_name = None;
    }

    pub fn is_logged_in(&self) -> bool {
        self.user_handle.is_some()
    }
}
