pub mod filesystem;
pub mod models;

#[tokio::main]
async fn main() {
    api::serve().await;
}

pub mod api {
    use crate::filesystem::{
        get_buf_reader_handle, get_buf_writer_handle, get_input_filepath, get_output_filepath,
    };
    use crate::keepalive_ops;
    use std::fs::{File, OpenOptions};
    use std::io::{BufReader, Read, Write};
    use std::thread;
    use std::time;
    use warp::Filter;

    pub async fn serve() -> () {
        keepalive_ops::start();

        let info = warp::path!("info").map(|| {
            let info_str = handle_info_fn();
            dbg!(&info_str);
            warp::reply::html(info_str)
        });

        let sleep = warp::path!("sleep").map(|| {
            handle_sleep();
            warp::reply::html("sleep")
        });
        let echo = warp::path!("echo").map(|| warp::reply::html("echo"));

        let routes = info.or(echo).or(sleep);

        warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
    }

    fn handle_sleep() {
        let mut writer = OpenOptions::new()
            .write(true)
            .append(true)
            .open(&get_input_filepath())
            .expect("should be able to get writer");

        println!("writing info");

        writer.write("sleep".as_bytes()).expect("should write good");
        writer.flush().expect("should flush");
    }

    fn handle_info_fn() -> String {
        let mut reader =
            get_buf_reader_handle(&get_output_filepath()).expect("should be able to get reader");
        // let mut writer = get_buf_writer_handle(&get_input_filepath()).expect("should be able to get writer");
        let mut writer = OpenOptions::new()
            .write(true)
            .append(true)
            .open(&get_input_filepath())
            .expect("should be able to get writer");

        println!("writing info");

        writer.write("info".as_bytes()).expect("should write good");
        writer.flush().expect("should flush");

        let mut buf_str = String::new();
        loop {
            {
                let file = File::open(get_output_filepath()).expect("should get file");
                buf_str.clear();
                BufReader::new(file)
                    .read_to_string(&mut buf_str)
                    .expect("should read good");
            }

            if buf_str.len() != 0 {
                break;
            }

            println!("sleeping...");
            thread::sleep(time::Duration::from_millis(100));
        }

        buf_str
    }
}

pub mod keepalive_ops {
    use crate::filesystem::get_process_filepath;
    use std::process::Command;

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
