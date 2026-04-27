#![allow(dead_code)]

use crate::bbs::data::Board;

pub struct BoardStore {
    pub boards: Vec<Board>,
}

impl BoardStore {
    pub fn new() -> Self {
        Self { boards: vec![] }
    }

    pub fn get_board(&self, id: &str) -> Option<&Board> {
        self.boards.iter().find(|b| b.id == id)
    }
}
