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
    #[error("not found error 404")]
    NotFound,
    #[error("cmd error")]
    CmdError(Id),
    #[error("parse cmd error")]
    ParseError(String),
    #[error("db error")]
    DbError,
    #[error("server error 500")]
    ServerError,
    #[error("input error 4XX")]
    InputError,
}

pub mod db {
    pub mod common {
        use std::collections::HashMap;

        pub type Id = String;

        pub type Metadata = HashMap<String, String>;

        pub trait HasId {
            fn get_id(&self) -> &Id;
        }
    }
    pub mod commands {
        use super::common::{HasId, Id};
        use serde::{Deserialize, Serialize};

        #[derive(Serialize, Deserialize, Debug)]
        pub struct Command {
            pub status: CommandStatus,
            pub args: Option<String>,
            pub name: CommandNames,
            pub issuer_id: Id,
            pub device_id: Id,
            _id: Id,
        }

        impl Default for Command {
            fn default() -> Self {
                Command {
                    status: CommandStatus::Pending,
                    args: None,
                    name: CommandNames::Update,
                    issuer_id: "default".to_string(),
                    device_id: "default".to_string(),
                    _id: "default".to_string(),
                }
            }
        }

        impl HasId for Command {
            fn get_id(&self) -> &Id {
                &self._id
            }
        }

        #[derive(Serialize, Deserialize, Debug)]
        pub enum CommandNames {
            Update,
            Test,
            ShellCmd,
        }

        #[derive(Serialize, Deserialize, Debug)]
        pub enum CommandStatus {
            Running,
            Blocked,
            Terminated,
            Failed,
            Ready,
            Pending,
            Sent,
            Received,
        }

        impl Default for CommandStatus {
            fn default() -> Self {
                Self::Received
            }
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
