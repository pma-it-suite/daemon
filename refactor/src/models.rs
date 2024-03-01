use serde::{Deserialize, Serialize};
use thiserror::Error;

use self::db::common::Id;

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

pub mod db {
    pub mod common {
        use std::collections::HashMap;

        pub type Id = String;

        pub type Metadata = HashMap<String, String>;
    }
    pub mod commands {
        use super::common::Id;
        use serde::{Deserialize, Serialize};

        #[derive(Serialize, Deserialize, Debug)]
        pub struct Command {
            pub status: CommandStatus,
            pub args: Option<String>,
            pub name: CommandNames,
            pub issuer_id: Id,
            pub device_id: Id,
        }

        #[derive(Serialize, Deserialize, Debug)]
        pub enum CommandNames {
            Update,
        }

        #[derive(Serialize, Deserialize, Debug)]
        pub enum CommandStatus {
            Running,
            Blocked,
            Terminated,
            Failed,
            Ready,
            Pending,
        }
    }

    pub mod devices {
        use super::common::{Id, Metadata};
        use serde::{Deserialize, Serialize};

        #[derive(Serialize, Deserialize, Debug)]
        pub struct Device {
            pub name: String,
            pub user_id: Id,
            pub command_ids: Vec<Id>,
            pub metadata: Option<Metadata>,
        }
    }
}