pub mod filesystem;
pub mod models;

#[tokio::main]
async fn main() {
    api::serve().await;
}

pub mod api {
    use crate::keepalive_ops;

    pub async fn serve() -> () {
        let process = keepalive_ops::start().expect("should be able to start keepalive");
        keepalive_ops::get_status(process.pid);
    }
}

pub mod keepalive_ops {
    use crate::filesystem::get_process_filepath;
    use crate::models::{FsResult, ProcessData, Status};
    use std::process::Command;
    use std::thread;
    use std::time;

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

    pub fn get_status(pid: u32) {

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
                println!("process data: {}", data);
            }
            None => println!("No process with PID {} found.", pid),
        }
    }

    fn _sleep(millis: u64) {
        let duration = time::Duration::from_millis(millis);
        thread::sleep(duration);
    }
}

pub mod os_access {
    pub fn get_info() {}
}
