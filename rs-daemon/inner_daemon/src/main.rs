use std::{thread, time};
pub mod filesystem;
pub mod models;
use models::{FullCmd, JsonStatus, InputCommands, HandlerError};

const WAIT_LONG: u64 = 2000;
const WAIT_SHORT: u64 = 2000;

#[tokio::main]
async fn main() -> Result<(), HandlerError> {
    println!("running inner daemon...");
    loop {
        if let Err(err) = os_ops::get_and_run_cmd().await {
            dbg!(&err);
            if matches!(&err, HandlerError::NotFound) {
                println!("no cmds found, sleeping...");
                sleep_blocking(WAIT_LONG);
            } else if matches!(&err, HandlerError::CmdError(_)) {
                println!("returning err: {}", &err);
                if let HandlerError::CmdError(cmd_id) = err {
                    let cmd = FullCmd { status: JsonStatus::Failed, id: cmd_id.to_string(), cmd: InputCommands::Info};
                    os_ops::update_status_for_cmd(&cmd).await?;
                }
            } else {
                println!("printing err: {}", &err);
            }
        }
        sleep_blocking(WAIT_SHORT);
    }
}

pub mod os_ops {
    use serde_json;
    use std::process::Command;
    use tokio::time::{sleep, Duration};
    use warp::Filter;
    use crate::models::{FullCmd, JsonStatus, InputCommands, HandlerError, JsonCmd};
    use crate::filesystem;

    pub async fn get_and_run_cmd() -> Result<(), HandlerError> {
        let mut full_cmd = fetch_cmds().await?;
        full_cmd.status = JsonStatus::InProgress;
        update_status_for_cmd(&full_cmd).await?;

        let resp = run_cmd(&full_cmd.cmd).await;
        if let Err(err) = resp {
            println!("issue with {:#?} : {}", &full_cmd, &err);
            return Err(HandlerError::CmdError(full_cmd.id));
        };

        if let Some(val) = resp.unwrap() {
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

    fn handle_shell_cmd(cmd_str: &str) -> Result<Option<String>, HandlerError> {
        let mut args = cmd_str.split_whitespace();
        let cmd_name = args.next().unwrap();
        let cmd = args
            .fold(&mut Command::new(cmd_name), |c, arg| c.arg(arg))
            .output()?;
        Ok(Some(String::from_utf8(cmd.stdout)?))
    }

    pub fn get_url() -> Result<String, HandlerError> {
        Ok(filesystem::deserialize_from_file()?.id.unwrap_or("https://its.kdns.ooo:5001".to_string()))
    }

    pub fn get_device_id() -> Result<String, HandlerError> {
        Ok(filesystem::deserialize_from_file()?.id.unwrap_or("b696b18b-c79f-48b7-b2d2-030d4c256402".to_string()))
    }

    pub fn get_user_id() -> Result<String, HandlerError> {
        Ok(filesystem::deserialize_from_file()?.id.unwrap())
    }

    pub async fn update_status_for_cmd(cmd: &FullCmd) -> Result<(), HandlerError> {
        let args = [
            ("command_id", cmd.id.clone()),
            ("status", cmd.status.to_output()),
        ];
        let url = get_url()? + "/commands/status";
        println!("updating...");
        dbg!(&url);
        dbg!(&args);
        reqwest::Client::new()
            .patch(url)
            .query(&args)
            .send()
            .await?;
        Ok(())
    }

    pub async fn fetch_cmds() -> Result<FullCmd, HandlerError> {
        let args = [("device_id", get_device_id()?)];
        let url = get_url()? + "/commands/recent";
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
        let input_resp = InputCommands::from(&(json_value.name, {
                let val = json_value.args;
                if val.is_empty() {
                    None
                } else {
                    Some(val)
                }
            }));

        let command_id = json_value.command_id;

        if let Err(HandlerError::ParseError(val)) = input_resp {
            println!("parse error on get cmds: {}", val);
            return Err(HandlerError::CmdError(command_id));
        }

        Ok(FullCmd {
            status: JsonStatus::from(&json_value.status),
            id: command_id,
            cmd: input_resp?,
        })
    }
    pub async fn serve() {
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

    fn get_info_str() -> Result<String, serde_json::Error> {
        Ok("".to_string())
    }
}

fn sleep_blocking(millis: u64) {
    let duration = time::Duration::from_millis(millis);
    thread::sleep(duration);
}
