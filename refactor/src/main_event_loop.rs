use log::{error, info, trace};
use std::error::Error;
use std::thread;
use std::time::Duration;

use crate::{
    api,
    models::{
        db::{
            commands::{Command, CommandStatus},
            common::Id,
        },
        HandlerError,
    },
};

use crate::main_event_loop::executor::handoff_command_to_executor;

const SLEEP_SHORT: u64 = 1;
const SLEEP_MEDIUM: u64 = 5;
const SLEEP_LONG: u64 = 10;

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
pub async fn run_main_event_loop(device_id: &Id, _user_id: &Id) -> Result<(), HandlerError> {
    loop {
        // get most recent command
        let command_resp = fetch_command(device_id).await;
        let sleep_int = match command_resp {
            Ok(Some(command)) => {
                dbg!(&command);
                let resp = update_command_status(&command, CommandStatus::Received).await;
                match resp {
                    Err(e) => {
                        handle_err(e);
                    }
                    Ok(_) => {
                        let resp = execute_command(&command).await;
                        let command_status = match resp {
                            Ok(data_opt) => {
                                info!("command executed is good, data: {:?}", data_opt);
                                CommandStatus::Terminated
                            }
                            Err(e) => {
                                handle_err(e);
                                CommandStatus::Failed
                            }
                        };

                        if let Err(e) = update_command_status(&command, command_status).await {
                            handle_err(e);
                        }
                    }
                }

                SLEEP_SHORT
            }
            Ok(None) => {
                info!("no commands found");
                SLEEP_MEDIUM
            }
            Err(e) => {
                handle_err(e);
                SLEEP_LONG
            }
        };

        sleep_in_seconds(sleep_int);
    }
}

pub async fn fetch_command(device_id: &Id) -> Result<Option<Command>, HandlerError> {
    let response = api::requests::fetch_commands::fetch_commands(device_id.clone()).await?;
    if response.is_none() {
        Ok(None)
    } else {
        let command = response.unwrap().command;
        Ok(Some(command))
    }
}

pub async fn update_command_status(
    command: &Command,
    new_status: CommandStatus,
) -> Result<(), HandlerError> {
    api::requests::update_command_status::update_command_status(command, new_status).await?;

    Ok(())
}

pub async fn execute_command(command: &Command) -> Result<Option<String>, HandlerError> {
    let resp = handoff_command_to_executor(command).await?;
    Ok(resp)
}

fn sleep_in_seconds(units: u64) {
    let sleep_in_ms = units * 1000;
    thread::sleep(Duration::from_millis(sleep_in_ms));
}

fn handle_err(err: HandlerError) {
    match err {
        HandlerError::ReqwestError(vals) => {
            trace!("ReqwestError: {:?}", &vals);
            let err_info = format!("url: {:#?}, source: {:#?}", vals.url(), vals.source());
            error!("Error in main loop: {}", &err_info);
        }
        _ => {
            dbg!(&err);
        }
    }
}

pub mod executor {
    use log::info;

    use crate::models::db::commands::{Command, CommandNames};
    use crate::models::HandlerError;

    pub async fn handoff_command_to_executor(
        command: &Command,
    ) -> Result<Option<String>, HandlerError> {
        info!("handing off command to executor: {:?}", &command);
        match command.name {
            CommandNames::Test => {
                // TODO @felipearce: add test command here
                Ok(Some("test".to_string()))
            }
            _ => {
                // TODO @felipearce: add more commands here
                Ok(None)
            }
        }
    }
}
