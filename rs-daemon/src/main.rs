pub mod filesystem;
pub mod models;

fn main() {
    keepalive_ops::start();
    println!("Hello, world!");
}

pub mod keepalive_ops {

    use std::io::prelude::*;
    use std::io::{BufReader, BufWriter};
    use std::io::Read;
    use std::process::{Command, Stdio};

    use std::{thread, time};
    use subprocess::{Popen, PopenConfig, Redirection};

    fn get_process_filepath() -> String {
        "/Users/felipearce/Desktop/projects/shellhacks2023/daemon/rs-daemon/inner_daemon/target/debug/inner_daemon".to_string()
    }

    fn get_input_filepath() -> String {
        "/Users/felipearce/Desktop/projects/shellhacks2023/daemon/rs-daemon/inner_daemon/out.txt".to_string()
    }

    fn get_output_filepath() -> String {
        "/Users/felipearce/Desktop/projects/shellhacks2023/daemon/rs-daemon/inner_daemon/out.txt".to_string()
    }

    pub fn start() {
        let mut child = Command::new(get_process_filepath())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("Failed to start subprocess.");

        // Optionally, you can wait for the child process to finish, if needed
        let status = child.wait().expect("Failed to wait for subprocess.");
        println!("Child process exited with: {:?}", status);
    }
    pub fn end() {}
    pub fn get_status() {}
}

pub mod os_access {
    pub fn get_info() {}
}
