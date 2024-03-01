use localstore::query_data;
use log::info;
use models::{
    db::{commands::Command, common::Id},
    HandlerError,
};

use crate::{localstore::write_single, models::db::commands::CommandStatus};

mod models;

#[tokio::main]
async fn main() -> Result<(), HandlerError> {
    std::env::set_var("RUST_LOG", "info");
    simple_logger::SimpleLogger::new().env().init().unwrap();

    // prelim check for data
    let user_id_key = "user_id";
    let user_id = query_data(&user_id_key)?.expect("user id to be set");
    info!("user_id retrieved from store is {}", user_id);

    // get device id or register it if not set
    let device_id_key = "device_id";
    let device_id_resp = query_data(&device_id_key);
    let device_id: Id;
    if (&device_id_resp).is_err() {
        device_id = register_device(&user_id).await?;
        write_single(&device_id, device_id_key)?;
    } else {
        device_id = device_id_resp?.unwrap();
    }
    info!("device id retrieved from store is {}", device_id);

    // get most recent command
    let command = fetch_commands(&device_id).await?;
    dbg!(&command);

    // update_command_status_received(command).await?;

    Ok(())
}

pub mod localstore {
    use crate::models::HandlerError;
    use jfs;
    use log::{info, warn};
    use std::collections::HashMap;
use lazy_static::lazy_static;
use std::sync::Mutex;

    fn get_file_path() -> String {
        "localstore.json".to_string()
    }

lazy_static! {
    static ref HANDLE: Mutex<Result<jfs::Store, HandlerError>> = Mutex::new(get_handle());
}

    fn get_handle() -> Result<jfs::Store, HandlerError> {
        let db = jfs::Store::new_with_cfg(
            get_file_path(),
            jfs::Config {
                single: true,
                pretty: true,
                ..Default::default()
            },
        )?;
        init_db(&db)?;
        Ok(db)
    }

    fn init_db(db: &jfs::Store) -> Result<(), HandlerError> {
        let key = "user_id";
        let resp = query_internal(db, &key);
        if resp.is_err() || resp?.is_none() {
            let user_id = "ee9470de-54a4-419c-b34a-ba2fa18731d8";
            db.save_with_id(&user_id.to_string(), &key)?;
            info!("user_id set to default: {}", user_id);
        }
        Ok(())
    }

    pub fn write_single(data: &String, key: &str) -> Result<(), HandlerError> {
        info!("writing data for key: {}", key);
        let binding = HANDLE.lock().unwrap();
        let handle = binding.as_ref().unwrap();
        handle.save_with_id(data, key)?;
        Ok(())
    }

    pub fn write_data(data: HashMap<String, String>) -> Result<(), HandlerError> {
        data.keys().for_each(|key| {
            write_single(&data[key], key).unwrap();
        });
        Ok(())
    }

    fn query_internal(db: &jfs::Store, key: &str) -> Result<Option<String>, HandlerError> {
        let data = db.get(key)?;
        Ok(data)
    }

    pub fn query_data(key: &str) -> Result<Option<String>, HandlerError> {
        info!("querying data for key: {}", key);
        let binding = HANDLE.lock().unwrap();
        let handle = binding.as_ref().unwrap();
        query_internal(&handle, key)
    }
}

/**
 * main (pre-registered) run loop:
 * 1. register device with server
 * 2. test connection to server
 */
pub fn get_device_name() -> String {
    "testdevicefelipearce".to_string()
}

pub async fn register_device(user_id: &Id) -> Result<Id, HandlerError> {
    let device_name = get_device_name();
    info!("registering device with name: {}", device_name);

    Ok(
        api::requests::register_device::register_device(user_id, device_name)
            .await?
            .device_id,
    )
}

pub async fn test_connection() -> Result<(), HandlerError> {
    unimplemented!()
}

/**
 * main (post-registered) run loop:
 * 1. call server to fetch commands using the deviceId (TODO @felipearce: add some auth eventually)
 *      a. if no commands found:
 *          - sleep for foobar seconds and then redo loop
 *
 * 2. call server to update command status as executing/etc. and send ACK to server
 * 3. execute command
 * 4. call server to send outgoing update commands status request if success or err. or blocking or etc.
 * 5. return data from command (if any)
 */
pub fn run_main_event_loop() {
    unimplemented!()
}

pub async fn fetch_commands(device_id: &Id) -> Result<Option<Command>, HandlerError> {
    let response = api::requests::fetch_commands::fetch_commands(device_id.clone()).await?;
    if response.is_none() {
        return Ok(None);
    }
    else {
        let command = response.unwrap().command;
        Ok(Some(command))
    }
}

pub fn fetch_next_command() {
    unimplemented!()
}

pub fn ack_command_received() {
    unimplemented!()
}

pub async fn update_command_status_received(command: Command) -> Result<(), HandlerError> {
    let new_status = CommandStatus::Ready;
    api::requests::update_command_status::update_command_status(&command, new_status).await?;

    Ok(())
}

pub fn execute_command() {
    unimplemented!()
}

pub fn update_command_status_after_execution() {
    unimplemented!()
}

// no call needed to return data

/**
 * impls
 */

pub mod api {

    pub mod request_models {
        pub mod update_command_status {
            use crate::models::db::{commands::CommandStatus, common::Id};
            use serde::{Deserialize, Serialize};

            #[derive(Serialize, Deserialize, Debug)]
            pub struct UpdateCommandStatusRequest {
                pub command_id: Id,
                pub status: CommandStatus,
            }
        }

        pub mod fetch_commands {
            use crate::models::db::{commands::Command, common::Id};
            use serde::{Deserialize, Serialize};

            #[derive(Serialize, Deserialize, Debug)]
            pub struct FetchRecentCommandResponse {
                pub command: Command,
            }
        }

        pub mod register_device {
            use crate::models::db::common::Id;
            use serde::{Deserialize, Serialize};

            #[derive(Serialize, Deserialize, Debug)]
            pub struct RegisterDeviceRequest {
                pub device_name: String,
                pub issuer_id: Id,
                pub user_id: Id,
            }

            #[derive(Serialize, Deserialize, Debug)]
            pub struct RegisterDeviceResponse {
                pub device_id: Id,
            }
        }
    }

    pub mod requests {
        use crate::models::HandlerError;
        pub type ApiResult<T> = Result<T, HandlerError>;

        fn get_port_string_if_any() -> String {
            "5001".to_string()
        }

        fn get_host() -> String {
            let port = get_port_string_if_any();
            format!("http://localhost:{}", port)
        }

        fn get_client() -> reqwest::Client {
            reqwest::Client::new()
        }

        pub mod register_device {
            use crate::api::request_models::register_device::{
                RegisterDeviceRequest, RegisterDeviceResponse,
            };
            use crate::api::requests::{get_client, get_host, ApiResult};
            use crate::models::db::common::Id;

            fn get_url() -> String {
                let host = get_host();
                format!("{}/devices/register", host)
            }

            pub async fn register_device(
                user_id: &Id,
                device_name: String,
            ) -> ApiResult<RegisterDeviceResponse> {
                let request = RegisterDeviceRequest {
                    user_id: user_id.clone(),
                    device_name,
                    issuer_id: user_id.clone(),
                };

                let url = get_url();

                let response = get_client().post(url).json(&request).send().await?;

                let status = response.status();
                println!("Response status: {}", status);

                let json = response.json().await?;
                Ok(json)
            }
        }

        pub mod update_command_status {
            use crate::api::request_models::update_command_status::UpdateCommandStatusRequest;
            use crate::api::requests::{get_client, get_host, ApiResult};
            use crate::models::db::commands::{Command, CommandStatus};
            use crate::models::db::common::HasId;

            fn get_url() -> String {
                let host = get_host();
                format!("{}/commands/update/status", host)
            }

            pub async fn update_command_status(
                command: &Command,
                new_status: CommandStatus,
            ) -> ApiResult<()> {
                let request = UpdateCommandStatusRequest {
                    command_id: command.get_id().clone(),
                    status: new_status,
                };

                let url = get_url();

                let response = get_client().patch(url).json(&request).send().await?;

                let status = response.status();
                println!("Response status: {}", status);

                let text = response.text().await?;
                println!("Response text: {}", text);

                Ok(())
            }
        }

        pub mod fetch_commands {
            use log::{info, warn};

            use crate::api::request_models::fetch_commands::FetchRecentCommandResponse;
            use crate::api::requests::{get_client, get_host, ApiResult};
            use crate::models::db::common::Id;

            fn get_url() -> String {
                let host = get_host();
                format!("{}/commands/recent", host)
            }

            pub async fn fetch_commands(device_id: Id) -> ApiResult<Option<FetchRecentCommandResponse>> {
                let url = get_url();

                let response = get_client()
                    .get(url)
                    .query(&[("device_id", device_id)])
                    .send()
                    .await?;

                let status = response.status();
                info!("Response status for fetch commands: {}", status);

                if status != 200 {
                    warn!("No commands found: {}", response.text().await?);
                    return Ok(None);
                }

                let json = response.json().await?;
                Ok(Some(json))
            }
        }
    }
}
