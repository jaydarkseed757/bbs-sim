#![allow(dead_code)]

use crate::bbs::data::Message;

pub struct MailBox {
    pub messages: Vec<Message>,
}

impl MailBox {
    pub fn new() -> Self {
        Self { messages: vec![] }
    }
}
