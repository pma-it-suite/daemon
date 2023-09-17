pub mod filesystem;
pub mod models;
use std::thread;
use std::time;

fn main() {
    let mut process = keepalive_ops::start().expect("should be able to start keepalive");
    loop {
        match keepalive_ops::get_status(&process) {
            Ok(models::Status::Alive) => {
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

pub mod keepalive_ops {
    use crate::filesystem::{self};
    use crate::models::{FsResult, ProcessData, Status};
    use std::fs;
    use std::path::Path;
    use std::process::Command;

    pub type KeepAliveResult<T> = FsResult<T>;

    fn get_tmp_filepath() -> String {
        "/usr/local/bin/inner_daemon".to_string()
    }

    pub fn start() -> KeepAliveResult<ProcessData> {
        let filepath = get_tmp_filepath();
        let mut child = Command::new(filepath)
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

        let process = ProcessData {
            pid: child.id(),
            status: status.clone(),
        };

        if status == Status::Alive {
            filesystem::save_process(&process)?;
        }

        Ok(process)
    }
    pub fn end() {}

    pub fn get_status(process: &ProcessData) -> KeepAliveResult<Status> {
        let pid = process.pid;
        let output = Command::new("ps")
            .arg("-p")
            .arg(pid.to_string())
            .arg("-o")
            .arg("pid,vsz,cpu,comm")
            .output()
            .expect("Failed to execute command");

        let output_str = {
            if output.status.success() {
                Some(String::from_utf8_lossy(&output.stdout).to_string())
            } else {
                None
            }
        };

        let status = match output_str {
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
        };

        if status != process.status {
            let new_process_data = ProcessData {
                pid,
                status: status.clone(),
            };
            filesystem::save_process(&new_process_data)?;
        }
        Ok(status)
    }
}

pub mod os_access {
    pub fn get_info() {}
}

fn sleep(millis: u64) {
    let duration = time::Duration::from_millis(millis);
    thread::sleep(duration);
}
