pub mod filesystem;
pub mod models;
use std::thread;
use std::time;

fn main() {
    let mut process = keepalive_ops::start().expect("should be able to start keepalive");
    loop {
        match keepalive_ops::get_status(process.pid) {
            models::Status::Alive => {
                println!("Sleeping...");
                sleep(10000);
            }
            _ => {
                println!("Restarting...");
                process = keepalive_ops::start().expect("should be able to start keepalive");
                sleep(100);
            }
        }
    }
}

pub mod api {
    use crate::keepalive_ops;

    pub fn _serve() -> () {
        let process = keepalive_ops::start().expect("should be able to start keepalive");
        let status = keepalive_ops::get_status(process.pid);
        dbg!(status);
    }
}

pub mod keepalive_ops {
    use crate::filesystem::get_process_filepath;
    use crate::models::{FsResult, ProcessData, Status};
    use std::process::Command;

    pub type KeepAliveResult<T> = FsResult<T>;

    pub fn start() -> KeepAliveResult<ProcessData> {
        let mut child = Command::new(get_process_filepath())
            .spawn()
            .expect("Failed to start subprocess.");

        // Optionally, you can wait for the child process to finish, if needed
        dbg!(&child.id());
        let status = match child.try_wait() {
            Ok(Some(status)) => {
                println!("process dead, status: {:#?}", &status);
                Status::Dead(status.to_string())
            }
            Ok(None) => Status::Alive,
            Err(err) => {
                println!("process broken, err: {:#?}", &err);
                Status::Broken(err.to_string())
            }
        };

        Ok(ProcessData {
            pid: child.id(),
            status,
        })
    }
    pub fn end() {}

    pub fn get_status(pid: u32) -> Status {
        let output = Command::new("ps")
            .arg("-p")
            .arg(pid.to_string())
            .arg("-o")
            .arg("pid,vsz,cpu,comm")
            .output()
            .expect("Failed to execute command");

        /*
        let s: &str = "42";

        match s.parse::<usize>() {
        Ok(n) => println!("Parsed number: {}", n),
        Err(e) => println!("Failed to parse: {}", e),
        }
        */

        let status = {
            if output.status.success() {
                Some(String::from_utf8_lossy(&output.stdout).to_string())
            } else {
                None
            }
        };

        match status {
            Some(data) => {
                if data.contains("defunct") {
                    println!("process dead: {}", data);
                    Status::Dead("process has died".to_string())
                } else {
                    println!("process data: {}", data);
                    Status::Alive
                }
            }
            None => {
                println!("No process with PID {} found.", pid);
                Status::Dead("no process".to_string())
            }
        }
    }
}

pub mod os_access {
    pub fn get_info() {}
}

fn sleep(millis: u64) {
    let duration = time::Duration::from_millis(millis);
    thread::sleep(duration);
}
