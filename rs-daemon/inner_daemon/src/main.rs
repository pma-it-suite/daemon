use os_ops::HandlerError; use std::{thread, time};
// use std::time::{SystemTime, UNIX_EPOCH};

const WAIT_LONG: u64 = 4000;
const WAIT_SHORT: u64 = 50;

/*
fn get_secs_since() -> u64 {
    let now = SystemTime::now();
    let since_the_epoch = now.duration_since(UNIX_EPOCH).expect("Time went backwards");
    since_the_epoch.as_secs()
}

fn should_run_or_wait_millis() -> Option<u64> {
    let secs_since = get_secs_since() % (WAIT_LONG / 1000);
    if secs_since != 0 {
        Some(secs_since)
    } else {
        None
    }
}
*/

#[tokio::main]
async fn main() -> Result<(), HandlerError> {
    println!("running inner daemon...");
    loop {
        /*
        if let Some(val) = should_run_or_wait_millis() {
            println!("sleeping for {} millis", val);
            sleep_blocking(val);
        }
        */

        if let Err(err) = os_ops::get_and_run_cmd().await {
            dbg!(&err);
            if matches!(&err, HandlerError::NotFound) {
                println!("no cmds found, sleeping...");
                sleep_blocking(WAIT_LONG);
            } else {
                println!("returning err: {}", &err);
                return Err(err);
            }
        }
        sleep_blocking(WAIT_SHORT);
    }
}

pub mod os_ops {
    use serde::{Deserialize, Serialize};
    use serde_json;
    use std::process::Command;
    use sys_info::{cpu_num, cpu_speed, loadavg, mem_info, os_release, os_type};
    use thiserror::Error;
    use tokio::time::{sleep, Duration};
    use warp::Filter;

    pub async fn get_and_run_cmd() -> Result<(), HandlerError> {
        let mut full_cmd = fetch_cmds().await?;
        full_cmd.status = JsonStatus::InProgress;
        update_status_for_cmd(&full_cmd).await?;
        if let Some(val) = run_cmd(&full_cmd.cmd).await.expect("run cmd ok") {
            println!("{}", val);
            full_cmd.status = JsonStatus::Finished;
            update_status_for_cmd(&full_cmd).await?;
        }

        Ok(())
    }

    pub async fn run_cmd(cmd: &InputCommands) -> Result<Option<String>, HandlerError> {
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
        #[error("reqwest error")]
        ReqwestError(#[from] reqwest::Error),
        #[error("api client error")]
        ApiError,
        #[error("serde error")]
        SerError(#[from] serde_json::Error),
        #[error("unknown error")]
        Unknown,
        #[error("unknown error")]
        DecodingError(#[from] std::string::FromUtf8Error),
        #[error("404")]
        NotFound,
    }

    fn handle_shell_cmd(cmd_str: &str) -> Result<Option<String>, HandlerError> {
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
        "http://172.178.91.48:5001".to_string()
    }

    pub fn get_device_id() -> String {
        // "b696b18b-c79f-48b7-b2d2-030d4c256402".to_string()
        std::env::var("ITS_DEVICE_ID").unwrap_or("b696b18b-c79f-48b7-b2d2-030d4c256402".to_string())
    }

    pub type RawInputCommand = (String, Option<String>);

    pub enum JsonStatus {
        Pending,
        InProgress,
        Finished,
        Failed,
    }

    impl JsonStatus {
        pub fn from(raw: &str) -> Self {
            match raw {
                "pending" => Self::Pending,
                "in_progress" => Self::InProgress,
                "finished" => Self::Finished,
                "failed" => Self::Failed,
                _ => unimplemented!(),
            }
        }

        pub fn to_output(&self) -> String {
            match self {
                Self::Pending => "pending",
                Self::InProgress => "in_progress",
                Self::Finished => "finished",
                Self::Failed => "failed",
            }
            .to_string()
        }
    }

    type Id = String;

    pub struct FullCmd {
        pub status: JsonStatus,
        pub id: Id,
        pub cmd: InputCommands,
    }

    #[derive(Serialize, Deserialize)]
    pub struct JsonCmd {
        pub status: String,
        pub command_id: Id,
        pub name: String,
        pub args: String,
    }

    pub async fn update_status_for_cmd(cmd: &FullCmd) -> Result<(), reqwest::Error> {
        let args = [
            ("command_id", cmd.id.clone()),
            ("status", cmd.status.to_output()),
        ];
        let url = get_url() + "/commands/status";
        reqwest::Client::new()
            .patch(url)
            .query(&args)
            .send()
            .await?;
        Ok(())
    }

    pub async fn fetch_cmds() -> Result<FullCmd, HandlerError> {
        let args = [("device_id", get_device_id())];
        let url = get_url() + "/commands/recent";
        let response = reqwest::Client::new().get(url).query(&args).send().await?;

        if response.status().is_client_error() || response.status().is_server_error() {
            match response.status() {
                reqwest::StatusCode::NOT_FOUND => {
                    // Handle 404 specifically here if needed
                    return Err(HandlerError::NotFound);
                }
                s if s.is_client_error() => {
                    // Handle general 4xx errors here
                    return Err(HandlerError::ApiError);
                }
                s if s.is_server_error() => {
                    // Handle general 5xx errors here
                    return Err(HandlerError::ApiError);
                }
                _ => {
                    // This should never hit since we're already inside the if condition checking for errors,
                    // but it's good to have a catch-all.
                    return Err(HandlerError::Unknown);
                }
            }
        }

        let body = response.text().await?;
        println!("Response:\n{}", body);

        let json_value = serde_json::from_str::<JsonCmd>(&body).expect("Failed to parse JSON");

        // Extract the "name" key's value as a string
        Ok(FullCmd {
            status: JsonStatus::from(&json_value.status),
            id: json_value.command_id,
            cmd: InputCommands::from(&(json_value.name, {
                let val = json_value.args;
                if val.is_empty() {
                    None
                } else {
                    Some(val)
                }
            })),
        })
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
