pub mod filesystem;
pub mod models;

fn main() {
    keepalive_ops::start();
    println!("Hello, world!");
}

pub mod keepalive_ops {
    use crate::filesystem;
    use crate::models::ProcessData;
    use std::fs::File;
    use std::path::Path;
    use std::{thread, time};
    use subprocess::{Popen, PopenConfig, Redirection};
    use std::io::Read;
    use std::io::Write;

    fn get_process_filepath() -> String {
        "/Users/felipearce/Desktop/projects/shellhacks2023/daemon/rs-daemon/inner_daemon/target/debug/inner_daemon".to_string()
    }

    pub fn start() {
        let file_path_str = "test.txt";
        let file_path = Path::new(file_path_str);

        let file = match file_path.exists() {
            true => File::open(file_path_str).unwrap(),
            false => File::create(file_path_str).unwrap(),
        };

        let in_file_path_str = "test_in.txt";
        let in_file_path = Path::new(in_file_path_str);

        let in_file = match in_file_path.exists() {
            true => File::open(in_file_path_str).unwrap(),
            false => File::create(in_file_path_str).unwrap(),
        };

        let err_file_path_str = "test_err.txt";
        let err_file_path = Path::new(err_file_path_str);

        let err_file = match err_file_path.exists() {
            true => File::open(err_file_path_str).unwrap(),
            false => File::create(err_file_path_str).unwrap(),
        };

        let mut p = Popen::create(
            &[&get_process_filepath()],
            PopenConfig {
                stdin: Redirection::File(in_file),
                stdout: Redirection::File(file),
                stderr: Redirection::File(err_file),
                ..Default::default()
            },
        )
        .unwrap();

        // Obtain the output from the standard streams.
        let (out, err) = p.communicate(None).unwrap();
        println!("stdout is: {:#?}", out);
        println!("err is: {:#?}", err);

        println!("pid is {:#?}", p.pid());


        let millis = 1500;
        let duration = time::Duration::from_millis(millis);

        for i in 0..100 {
            if let Some(exit_status) = p.poll() {
                // the process has finished
                println!("stdout is: {:#?}", out);
                println!("exited with {:#?}", exit_status);
                let mut output = "".to_string();
                // let mut out_file = File::open(file_path_str).unwrap();
                let mut out_file = p.stdout.as_mut().unwrap();
                out_file.read_to_string(&mut output).unwrap();
                println!("output is: {}", &output);
            } else {
                let content = format!("in step: {}", i);
                let mut in_file = p.stdin.as_mut().unwrap();
                in_file.write(content.as_bytes()).unwrap();
                println!("sleeping...");
                thread::sleep(duration);
            }
        }
        println!("terminating");
        p.terminate().unwrap();
    }
    pub fn end() {}
    pub fn get_status() {}
}

pub mod os_access {
    pub fn get_info() {}
}

