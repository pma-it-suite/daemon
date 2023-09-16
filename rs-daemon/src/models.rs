use serde::{Deserialize, Serialize};
use serde_json;
use std::fmt;
pub type FsResult<T> = Result<T, std::io::Error>;

#[derive(Serialize, Deserialize)]
pub enum Status {
    Alive,
    Dead,
    Broken,
    Deploying,
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Status::Alive => write!(f, "Alive"),
            Status::Dead => write!(f, "Dead"),
            Status::Broken => write!(f, "Broken"),
            Status::Deploying => write!(f, "Deploying"),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct ProcessData {
    pid: String,
    status: Status,
}

impl ProcessData {
    pub fn to_output(&self) -> String {
        serde_json::to_string(self).expect("should serialize process data always")
    }
}
