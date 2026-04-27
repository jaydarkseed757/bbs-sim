use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BbsEntry {
    pub name: String,
    pub number: String,
    pub sysop: String,
    pub location: String,
    pub baud: u32,
    pub boards: Vec<String>,
    #[serde(default)]
    pub last_called: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub handle: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: u32,
    pub author: String,
    pub subject: String,
    pub body: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Thread {
    pub id: u32,
    pub subject: String,
    pub posts: Vec<Message>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Board {
    pub id: String,
    pub name: String,
    pub threads: Vec<Thread>,
}
