use serde::{Deserialize, Serialize};
use serde_json;
use std::fmt;
pub type FsResult<T> = Result<T, std::io::Error>;

#[derive(Serialize, Deserialize)]
pub enum Status {
    Alive,
    Dead(String),
    Broken(String),
    Deploying,
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Status::Alive => write!(f, "Alive"),
            Status::Dead(why) => write!(f, "Dead: {}", why),
            Status::Broken(why) => write!(f, "Broken: {}", why),
            Status::Deploying => write!(f, "Deploying"),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct ProcessData {
    pub pid: u32,
    pub status: Status,
}

impl ProcessData {
    pub fn to_output(&self) -> String {
        serde_json::to_string(self).expect("should serialize process data always")
    }
}
