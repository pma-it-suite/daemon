pub mod filesystem;
pub mod models;

#[tokio::main]
async fn main() {
    api::serve().await;
}

pub mod api {
    use crate::keepalive_ops;
    use std::thread;
    use std::time;

    pub async fn serve() -> () {
        keepalive_ops::start();
    }
}

pub mod keepalive_ops {
    use crate::filesystem::get_process_filepath;
    use std::process::Command;

    pub fn start() {
        // Command::new(get_process_filepath())
        Command::new("echo")
            .spawn()
            .expect("Failed to start subprocess.");

        // Optionally, you can wait for the child process to finish, if needed
        // let status = child.wait().expect("Failed to wait for subprocess.");
        // println!("Child process exited with: {:?}", "fail");
    }
    pub fn end() {}
    pub fn get_status() {}
}

pub mod os_access {
    pub fn get_info() {}
}
