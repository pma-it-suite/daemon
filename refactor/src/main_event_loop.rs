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
    // get most recent command
    let command = fetch_commands(device_id).await?;
    dbg!(&command);

    // update_command_status_received(command).await?;
    unimplemented!()
}

pub async fn fetch_commands(device_id: &Id) -> Result<Option<Command>, HandlerError> {
    let response = api::requests::fetch_commands::fetch_commands(device_id.clone()).await?;
    if response.is_none() {
        Ok(None)
    } else {
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
