fn main() {
    println!("Hello, world!");
    os_ops::poll_from_stdin();
}

pub mod os_ops {
    use serde::{Deserialize, Serialize};
    use serde_json;
    use std::fs::File;
    use std::io::Read;
    use std::io::Write;
    use std::io::{self, BufReader, BufWriter};
    use std::path::Path;
    use std::process::Command;
    use std::{thread, time};
    use sys_info::{cpu_num, cpu_speed, loadavg, mem_info, os_release, os_type};

    fn post_and_flush(content: &str) {
        println!("{}", content);
        io::stdout().flush().unwrap();
    }

    pub fn poll_from_stdin() {
        let mut reader = get_buf_reader_handle().expect("should get buf reader handle");
        let mut output_writer =
            get_buf_writer_handle(get_output_file()).expect("should get output writer handle");

        let mut can_delete = true;

        loop {
            if can_delete == false {
                reader = BufReader::new(
                    File::open(get_input_file()).expect("should be able to refresh buf"),
                );
            }
            let mut bug_str = String::new();
            match reader.read_to_string(&mut bug_str) {
                Ok(len) => {
                    if len == 0 {
                        println!("can to delete...: {}", &can_delete);
                        if can_delete {
                            File::create(get_input_file())
                                .expect("should be able to wipe input file");
                            // input_writer.write_all("".as_bytes()).unwrap();
                            // input_writer.flush().unwrap();
                            can_delete = false;
                        }
                    } else {
                        post_and_flush(&format!("Received: {} | len: {}", bug_str, len));
                        post_and_flush("executing cmd...");
                        let output = match_input_to_output(&bug_str.trim());
                        post_and_flush(&format!("going to output : {}", &output));
                        {
                            let mut output_writer = BufWriter::new(
                                File::create(get_output_file())
                                    .expect("should be able to get writer"),
                            );
                            output_writer
                                .write_all(output.as_bytes())
                                .expect("should write out good");
                            output_writer.flush().expect("should flush the output");
                        }
                        can_delete = true;
                    }
                }
                Err(err) => panic!("Channel disconnected, {:#?}", err),
            }
            sleep(1000);
        }
    }

    fn match_input_to_output(input: &str) -> String {
        dbg!(&input);
        match input {
            "info" => get_info_str().unwrap(),
            "sleep" => {
                Command::new("osascript")
                    .arg("-e")
                    .arg(r#"tell app "System Events" to sleep"#)
                    .output()
                    .expect("Failed to send sleep command");
                "sleep".to_string()
            }
            "echo" => "echo".to_string(),
            _ => "none".to_string(),
        }
    }

    pub fn get_input_file() -> String {
        "/Users/felipearce/Desktop/projects/shellhacks2023/daemon/rs-daemon/inner_daemon/in.txt"
            .to_string()
    }

    pub fn get_output_file() -> String {
        "/Users/felipearce/Desktop/projects/shellhacks2023/daemon/rs-daemon/inner_daemon/out.txt"
            .to_string()
    }

    fn get_buf_reader_handle() -> io::Result<BufReader<File>> {
        let file = get_or_create(&get_input_file())?;
        Ok(BufReader::new(file))
    }

    fn get_buf_writer_handle(filepath: String) -> io::Result<BufWriter<File>> {
        let file = get_or_create(&filepath)?;
        Ok(BufWriter::new(file))
    }

    fn get_or_create(file_path_str: &str) -> io::Result<File> {
        let file_path = Path::new(file_path_str);

        let file = match file_path.exists() {
            true => File::open(file_path_str)?,
            false => File::create(file_path_str)?,
        };
        Ok(file)
    }

    fn sleep(millis: u64) {
        let duration = time::Duration::from_millis(millis);
        thread::sleep(duration);
    }

    #[derive(Serialize, Deserialize)]
    struct SystemInfo {
        cpu_count: u32,
        cpu_speed: Option<u64>,
        load_avg: (f64, f64, f64),
        mem_total: u64,
        mem_free: u64,
        os_type: String,
        os_release: String,
    }

    fn get_info_str() -> Result<String, serde_json::Error> {
        let cpu_count = cpu_num().unwrap_or(0);
        let cpu_speed = cpu_speed().ok();
        let load_avg_result = loadavg();
        let load_avg = if let Ok(la) = load_avg_result {
            (la.one, la.five, la.fifteen)
        } else {
            (0.0, 0.0, 0.0)
        };
        let mem_result = mem_info();
        let (mem_total, mem_free) = if let Ok(mem) = mem_result {
            (mem.total, mem.free)
        } else {
            (0, 0)
        };

        let os_type = os_type().unwrap_or_else(|_| "".to_string());
        let os_release = os_release().unwrap_or_else(|_| "".to_string());

        let system_info = SystemInfo {
            cpu_count,
            cpu_speed,
            load_avg,
            mem_total,
            mem_free,
            os_type,
            os_release,
        };

        let json_str = serde_json::to_string(&system_info)?;
        Ok(format!("{}", json_str))
    }
}
