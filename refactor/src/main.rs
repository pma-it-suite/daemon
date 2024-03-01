use models::{db::commands::Command, HandlerError};

use crate::models::db::commands::CommandStatus;

mod models;

#[tokio::main]
async fn main() -> Result<(), HandlerError> {
    println!("Hello, world!");
    // update_command_status_received(command).await?;

    Ok(())
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

pub fn fetch_commands() {
    unimplemented!()
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
    }

    pub mod requests {
        use crate::models::HandlerError;
        pub type ApiResult<T> = Result<T, HandlerError>;

        pub mod update_command_status {
            use crate::api::request_models::update_command_status::UpdateCommandStatusRequest;
            use crate::api::requests::ApiResult;
            use crate::models::db::commands::{Command, CommandStatus};
            use crate::models::db::common::HasId;

            fn get_port_string_if_any() -> String {
                "5001".to_string()
            }

            fn get_host() -> String {
                let port = get_port_string_if_any();
                format!("http://localhost:{}", port)
            }

            fn get_url() -> String {
                let host = get_host();
                format!("{}/commands/update/status", host)
            }

            fn get_client() -> reqwest::Client {
                reqwest::Client::new()
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
    }
}
