pub mod filesystem;
pub mod models;

#[tokio::main]
async fn main() {
    api::serve().await;
}

pub mod api {
    use warp::Filter;
    use crate::keepalive_ops;
    use std::fs::OpenOptions;
    use crate::filesystem::{get_buf_reader_handle, get_buf_writer_handle, get_input_filepath, get_output_filepath};
    use std::io::{Read, Write};

    pub async fn serve() -> () {
        keepalive_ops::start();

        let info = warp::path("info").map(|| warp::reply::html(handle_info_fn()));
        let echo = warp::path("echo").map(|| warp::reply::html("echo"));

        let routes = info.or(echo);

        warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
    }

    fn handle_info_fn() -> String {
        let mut reader = get_buf_reader_handle(&get_output_filepath()).expect("should be able to get reader");
        // let mut writer = get_buf_writer_handle(&get_input_filepath()).expect("should be able to get writer");
        let mut writer = OpenOptions::new()
        .write(true)
        .append(true)
        .open(&get_input_filepath())
        .expect("should be able to get writer");

        writer.write("info".as_bytes()).expect("should write good");
        writer.flush().expect("should flush");
        let mut buf_str = String::new();
        reader.read_to_string(&mut buf_str).expect("should read good");

        buf_str
    }
}

pub mod keepalive_ops {
    use std::process::{Command};
    use crate::filesystem::get_process_filepath;

    pub fn start() {
        Command::new(get_process_filepath())
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
