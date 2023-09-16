pub mod filesystem;
pub mod models;

fn main() {
    keepalive_ops::start();
    println!("Hello, world!");
}

pub mod keepalive_ops {

    use std::io::prelude::*;
    use std::io::BufReader;
    use std::io::Read;
    use std::process::{Command, Stdio};

    use std::{thread, time};
    use subprocess::{Popen, PopenConfig, Redirection};

    fn get_process_filepath() -> String {
        "/Users/felipearce/Desktop/projects/shellhacks2023/daemon/rs-daemon/inner_daemon/target/debug/inner_daemon".to_string()
    }

    pub fn start() {
        let mut child = Command::new(get_process_filepath())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("Failed to start subprocess.");

        // Assuming you want to write to the child's stdin.
        {
            let child_stdin = child.stdin.as_mut().expect("Failed to get stdin handle.");
            child_stdin
                .write_all(b"Your data to child")
                .expect("Failed to write to stdin.");
        }

        {
            // Using a BufReader to read lines from the child's stdout.
            let child_stdout = child.stdout.as_mut().expect("Failed to get stdout handle.");
            let reader = BufReader::new(child_stdout);

            // Read lines from stdout as they're written by the subprocess
            for line in reader.lines() {
                let line = line.expect("Failed to read a line from subprocess.");
                println!("Received: {}", line);
            }
        }

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
