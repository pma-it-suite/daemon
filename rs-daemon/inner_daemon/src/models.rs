use serde::{Deserialize, Serialize};
use thiserror::Error;

pub type RawInputCommand = (String, Option<String>);

#[derive(Serialize, Deserialize, Debug)]
pub struct DeviceData {
    pub id: String,
    pub user_id: String,
    pub user_secret: String,
    pub endpoint: String,
}

pub type Id = String;

#[derive(Debug)]
pub struct FullCmd {
    pub status: JsonStatus,
    pub id: Id,
    pub cmd: InputCommands,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JsonCmd {
    pub status: String,
    pub command_id: Id,
    pub name: String,
    pub args: String,
}

#[derive(Serialize, Deserialize)]
pub struct SystemInfo {
    cpu_count: u32,
    cpu_speed: Option<u64>,
    load_avg: (f64, f64, f64),
    mem_total: u64,
    mem_free: u64,
    os_type: String,
    os_release: String,
}

#[derive(Error, Debug)]
pub enum HandlerError {
    #[error("io error")]
    IoError(#[from] std::io::Error),
    #[error("reqwest error")]
    ReqwestError(#[from] reqwest::Error),
    #[error("api client error")]
    ApiError,
    #[error("serde error")]
    SerError(#[from] serde_json::Error),
    #[error("unknown error")]
    Unknown,
    #[error("unknown error")]
    DecodingError(#[from] std::string::FromUtf8Error),
    #[error("404")]
    NotFound,
    #[error("cmd error")]
    CmdError(Id),
    #[error("parse cmd error")]
    ParseError(String),
    #[error("db error")]
    DbError,
}

#[derive(Debug)]
pub enum InputCommands {
    Info,
    Sleep,
    Health,
    ShellCmd(String),
}

#[derive(Debug)]
pub enum JsonStatus {
    Pending,
    InProgress,
    Finished,
    Failed,
}

impl JsonStatus {
    pub fn from(raw: &str) -> Self {
        match raw {
            "pending" => Self::Pending,
            "in_progress" => Self::InProgress,
            "finished" => Self::Finished,
            "failed" => Self::Failed,
            _ => unimplemented!(),
        }
    }

    pub fn to_output(&self) -> String {
        match self {
            Self::Pending => "pending",
            Self::InProgress => "in_progress",
            Self::Finished => "finished",
            Self::Failed => "failed",
        }
        .to_string()
    }
}

impl InputCommands {
    pub fn from(raw_input: &RawInputCommand) -> Result<Self, HandlerError> {
        match (raw_input.0.as_str(), {
            raw_input.1.as_deref()
        }) {
            ("info", None) => Ok(Self::Info),
            ("sleep", None) => Ok(Self::Sleep),
            ("health", None) => Ok(Self::Health),
            ("shellCmd", Some(args)) => Ok(Self::ShellCmd(args.to_string())),
            ("shellCmd", None) => Err(HandlerError::ParseError(format!(
                "no args for cmd: {:#?}",
                &raw_input
            ))),
            _ => Err(HandlerError::ParseError(format!(
                "not implemented for input: {:#?}",
                &raw_input
            ))),
        }
    }
}
