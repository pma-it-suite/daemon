use std::{thread, time};
pub mod filesystem;
pub mod models;
use models::{FullCmd, HandlerError, InputCommands, JsonStatus};

const WAIT_LONG: u64 = 10000;
const WAIT_SHORT: u64 = 10000;

#[tokio::main]
async fn main() -> Result<(), HandlerError> {
    println!("running inner daemon...");
    loop {
        if let Err(err) = os_ops::get_and_run_cmd().await {
            dbg!(&err);
            if matches!(&err, HandlerError::NotFound) {
                println!("no cmds found, sleeping...");
                println!("----------------------------------------------");
                sleep_blocking(WAIT_LONG);
            } else if matches!(&err, HandlerError::CmdError(_)) {
                println!("returning err: {}", &err);
                if let HandlerError::CmdError(cmd_id) = err {
                    let cmd = FullCmd {
                        status: JsonStatus::Failed,
                        id: cmd_id.to_string(),
                        cmd: InputCommands::Info,
                    };
                    os_ops::update_status_for_cmd(&cmd).await?;
                }
            } else {
                println!("printing err: {}", &err);
            }
        }
        println!("----------------------------------------------");
        sleep_blocking(WAIT_SHORT);
    }
}

pub mod os_ops {
    use crate::filesystem;
    use crate::models::{
        DeviceData, FullCmd, HandlerError, Id, InputCommands, JsonCmd, JsonStatus,
    };
    use serde_json;
    use std::process::Command;
    use tokio::time::{sleep, Duration};
    use warp::Filter;

    pub async fn get_and_run_cmd() -> Result<(), HandlerError> {
        if let None = get_device_id()? {
            let id = send_register_device_call().await?;
            register_device(id)?;
        }
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

    pub fn register_device(id: Id) -> Result<(), HandlerError> {
        let mut data = get_device_data()?;
        data.id = id;
        print!("updating registering at... {:#?}", &data);
        filesystem::update_or_add_to_file(data)?;
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

    pub fn get_device_data() -> Result<DeviceData, HandlerError> {
        Ok(filesystem::deserialize_from_file()?)
    }

    pub fn get_empty_device_data() -> DeviceData {
        DeviceData {
            endpoint: "".to_string(),
            user_id: "".to_string(),
            user_secret: "".to_string(),
            id: "".to_string(),
        }
    }

    pub fn get_url() -> Result<String, HandlerError> {
        println!("getting url from db...");
        Ok(filesystem::deserialize_from_file()?.endpoint)
    }

    pub fn get_device_id() -> Result<Option<String>, HandlerError> {
        println!("getting device_id from db...");
        Ok({
            let opt = filesystem::deserialize_from_file()?.id;
            if opt.len() == 0 {
                None
            } else {
                Some(opt)
            }
        })
    }

    pub fn get_user_id() -> Result<String, HandlerError> {
        println!("getting user_id from db...");
        Ok(filesystem::deserialize_from_file()?.user_id)
    }

    pub fn get_user_secret() -> Result<String, HandlerError> {
        println!("getting secret from db...");
        Ok(filesystem::deserialize_from_file()?.user_secret)
    }

    pub async fn send_register_device_call() -> Result<Id, HandlerError> {
        let mut data = std::collections::HashMap::new();
        data.insert("user_id", get_user_id()?);
        data.insert("user_secret", get_user_secret()?);

        let url = get_url()? + "/devices/register";
        println!("registering...");
        dbg!(&url);
        dbg!(&data);
        let resp = reqwest::Client::new().post(url).json(&data).send().await?;

        handle_response(&resp)?;

        let body = resp.text().await?;

        let json_value =
            serde_json::from_str::<serde_json::Value>(&body).expect("Failed to parse JSON");
        let device_id = json_value["device_id"].to_string().replace("\\", "").replace("\"", "");
        dbg!(&device_id);
        Ok(device_id)
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
        let resp = reqwest::Client::new()
            .patch(url)
            .query(&args)
            .send()
            .await?;

        handle_response(&resp)?;
        Ok(())
    }

    pub fn handle_response(response: &reqwest::Response) -> Result<(), HandlerError> {
        if response.status().is_client_error() || response.status().is_server_error() {
            dbg!(&response);
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
        } else {
            Ok(())
        }
    }

    pub async fn fetch_cmds() -> Result<FullCmd, HandlerError> {
        let device_id = match get_device_id()? {
            Some(data) => data,
            None => return Err(HandlerError::DbError)
        };
        let args = [("device_id", device_id)];
        let url = get_url()? + "/commands/recent";

        dbg!(&url);
        dbg!(&args);

        let response = reqwest::Client::new().get(url).query(&args).send().await?;

        handle_response(&response)?;

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
