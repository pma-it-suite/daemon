use reqwest;
use std::{thread, time};

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    println!("running inner daemon...");
    loop {
        os_ops::get_and_run_cmd().await?;
        sleep_blocking(5000);
    }
}

pub mod os_ops {
    use serde::{Deserialize, Serialize};
    use serde_json;
    use std::process::Command;
    use sys_info::{cpu_num, cpu_speed, loadavg, mem_info, os_release, os_type};
    use tokio::time::{sleep, Duration};
    use warp::Filter;
    use thiserror::Error;


    pub async fn get_and_run_cmd() -> Result<(), reqwest::Error> {
        let raw_cmd = fetch_cmds().await?;
        let cmd = InputCommands::from(&raw_cmd);
        if let Some(val) = run_cmd(cmd).await.expect("run cmd ok") {
            println!("{}", val);
        }

        Ok(())
    }

    pub async fn run_cmd(cmd: InputCommands) -> Result<Option<String>, HandlerError> {
        match cmd {
            InputCommands::Info => Ok(Some(get_info_str()?)),
            InputCommands::Sleep => {
                {
                    sleep(Duration::from_secs(1)).await;
                    handle_sleep();
                }
                Ok(None)
            }
            InputCommands::Health => Ok(Some("ok".to_string())),
            InputCommands::ShellCmd(cmd_str) => handle_shell_cmd(cmd_str),
        }
    }

    #[derive(Error, Debug)]
    pub enum HandlerError {
        #[error("io error")]
        IoError(#[from] std::io::Error),
        #[error("serde error")]
        SerError(#[from] serde_json::Error),
        #[error("unknown error")]
        Unknown,
        #[error("unknown error")]
        DecodingError(#[from] std::string::FromUtf8Error),
    }

    fn handle_shell_cmd(cmd_str: String) -> Result<Option<String>, HandlerError> {
        let mut args = cmd_str.split_whitespace();
        let cmd_name = args.next().unwrap();
        let cmd = args
            .fold(&mut Command::new(cmd_name), |c, arg| c.arg(arg))
            .output()?;
        Ok(Some(String::from_utf8(cmd.stdout)?))
    }

    pub enum InputCommands {
        Info,
        Sleep,
        Health,
        ShellCmd(String),
    }

    impl InputCommands {
        pub fn from(raw_input: &RawInputCommand) -> Self {
            match (raw_input.0.as_str(), {
                match &raw_input.1 {
                    Some(val) => Some(val.as_str()),
                    None => None,
                }
            }) {
                ("info", None) => Self::Info,
                ("sleep", None) => Self::Sleep,
                ("health", None) => Self::Health,
                ("shellCmd", Some(args)) => Self::ShellCmd(args.to_string()),
                _ => panic!("not implemented for input: {:#?}", &raw_input),
            }
        }
    }

    pub fn get_url() -> String {
        "http://localhost:4040".to_string()
    }

    pub fn get_device_id() -> String {
        "mac01".to_string()
    }

    pub type RawInputCommand = (String, Option<String>);

    pub async fn fetch_cmds() -> Result<RawInputCommand, reqwest::Error> {
        let args = [("deviceId", get_device_id())];
        let url = get_url() + "/fetch";
        let response = reqwest::Client::new().get(url).query(&args).send().await?;
        let body = response.text().await?;
        println!("Response:\n{}", body);

        let json_value: serde_json::Value =
            serde_json::from_str(&body).expect("Failed to parse JSON");

        // Extract the "name" key's value as a string
        let cmd = (
            json_value["name"]
                .as_str()
                .expect("no name key in json")
                .to_string(),
            {
                let val = json_value["args"].as_str().expect("no args key in json");

                if val.len() == 0 {
                    None
                } else {
                    Some(val.to_string())
                }
            },
        );

        Ok(cmd)
    }

    pub async fn serve() -> () {
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
        Command::new("osascript")
            .arg("-e")
            .arg(r#"tell app "System Events" to sleep"#)
            .output()
            .expect("Failed to send sleep command");
    }

    fn handle_info_fn() -> String {
        get_info_str().expect("get info")
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

fn sleep_blocking(millis: u64) {
    let duration = time::Duration::from_millis(millis);
    thread::sleep(duration);
}
