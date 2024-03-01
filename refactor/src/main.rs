mod models;
fn main() {
    println!("Hello, world!");
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
pub fn run_main_event_loop() {}

pub fn fetch_commands() {}

pub fn fetch_next_command() {}

pub fn ack_command_received() {}

pub fn update_command_status_received() {}

pub fn execute_command() {}

pub fn update_command_status_after_execution() {}

// no call needed to return data


