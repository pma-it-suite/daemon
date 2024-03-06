use crate::models::HandlerError;
use futures::future::BoxFuture;
use log::{error, warn};
use reqwest::StatusCode;
pub type ApiResult<T> = Result<T, HandlerError>;

fn get_client() -> reqwest::Client {
    reqwest::Client::new()
}

pub struct ApiConfig {
    pub host: String,
    pub port: Option<u16>,
}

impl ApiConfig {
    pub fn new(host: String, port: Option<u16>) -> Self {
        ApiConfig { host, port }
    }

    fn get_port_string_if_any(&self) -> String {
        match self.port {
            Some(val) => format!(":{}", &val),
            None => "".to_string(),
        }
    }

    pub fn with_path(&self, path: &str) -> String {
        let port_string = self.get_port_string_if_any();
        format!("{}{}{}", self.host, port_string, path)
    }
}

impl Default for ApiConfig {
    fn default() -> Self {
        ApiConfig {
            host: "localhost".to_string(),
            port: Some(5001),
        }
    }
}

async fn handle_response<T>(
    response: reqwest::Response,
    on_ok: impl Fn(reqwest::Response) -> BoxFuture<'static, Result<T, HandlerError>>,
) -> Result<T, HandlerError> {
    let status = response.status();
    if let StatusCode::OK = status {
        Ok(on_ok(response).await?)
    } else {
        let text = response.text().await?;
        match status {
            StatusCode::NOT_FOUND => {
                warn!("No commands found: {}", &text);
                Err(HandlerError::NotFound)
            }
            StatusCode::INTERNAL_SERVER_ERROR => {
                error!("server error on fetch: {}", &text);
                Err(HandlerError::ServerError)
            }
            StatusCode::BAD_REQUEST | StatusCode::UNPROCESSABLE_ENTITY => {
                warn!("error in data passed in: {}", &text);
                Err(HandlerError::InputError)
            }
            _ => {
                warn!("unknown error code: {}, {}", &text, status);
                Err(HandlerError::ApiError)
            }
        }
    }
}

pub mod register_device {
    use futures::future::BoxFuture;

    use crate::api::models::register_device::{RegisterDeviceRequest, RegisterDeviceResponse};
    use crate::api::requests::{get_client, handle_response, ApiResult};
    use crate::models::db::common::Id;

    use super::ApiConfig;

    pub async fn register_device(
        user_id: &Id,
        device_name: String,
        config: &ApiConfig,
    ) -> ApiResult<RegisterDeviceResponse> {
        let request = RegisterDeviceRequest {
            user_id: user_id.clone(),
            device_name,
            issuer_id: user_id.clone(),
        };

        let url = config.with_path("/devices/register");

        let response = get_client().post(url).json(&request).send().await?;

        let bind =
            |response: reqwest::Response| -> BoxFuture<'static, ApiResult<RegisterDeviceResponse>> {
                Box::pin(async move { Ok(response.json().await?) })
            };
        handle_response(response, bind).await
    }

    #[cfg(test)]
    mod test {
        use crate::{
            api::models::register_device::{RegisterDeviceRequest, RegisterDeviceResponse}, models::{db::common::Id, HandlerError}, test_commons::{before_each, get_404_json_string, get_500_json_string, setup_server}
        };

        fn get_json_payload(device_id: Id) -> (RegisterDeviceResponse, String) {
            let data = RegisterDeviceResponse::new(device_id);
            let data_string = serde_json::to_string(&data).unwrap();

            (data, data_string)
        }

        #[tokio::test]
        async fn test_register_device() {
            before_each();

            let device_id = "testdeviceid".to_string();
            let (data, json) = get_json_payload(device_id);
            let (mut server, config) = setup_server();

            let mock = server
                .mock("POST", "/devices/register")
                .with_status(200)
                .with_body(json)
                .create();

            let input = RegisterDeviceRequest::default();
            let result = super::register_device(&input.user_id, input.device_name, &config).await;

            assert!(result.is_ok());
            let response = result.unwrap();
            assert_eq!(response.device_id, data.device_id);
            mock.assert();
        }

        #[tokio::test]
        async fn test_register_device_404_fail() {
            before_each();

            let (mut server, config) = setup_server();

            let mock = server
                .mock("POST", "/devices/register")
                .with_status(404)
                .with_body(get_404_json_string())
                .create();

            let input = RegisterDeviceRequest::default();
            let result = super::register_device(&input.user_id, input.device_name, &config).await;

            assert!(result.is_err());
            assert!(matches!(result.err().unwrap(), HandlerError::NotFound));
            mock.assert();
        }

        #[tokio::test]
        async fn test_register_device_500_fail() {
            before_each();

            let (mut server, config) = setup_server();

            let mock = server
                .mock("POST", "/devices/register")
                .with_status(500)
                .with_body(get_500_json_string())
                .create();

            let input = RegisterDeviceRequest::default();
            let result = super::register_device(&input.user_id, input.device_name, &config).await;

            assert!(result.is_err());
            assert!(matches!(result.err().unwrap(), HandlerError::ServerError));
            mock.assert();
        }
    }
}

pub mod update_command_status {
    use futures::future::BoxFuture;

    use crate::api::models::update_command_status::UpdateCommandStatusRequest;
    use crate::api::requests::{get_client, ApiResult};
    use crate::models::db::commands::{Command, CommandStatus};
    use crate::models::db::common::HasId;

    use super::{handle_response, ApiConfig};

    pub async fn update_command_status(
        command: &Command,
        new_status: CommandStatus,
        config: &ApiConfig,
    ) -> ApiResult<()> {
        let request = UpdateCommandStatusRequest {
            command_id: command.get_id().clone(),
            status: new_status,
        };

        let url = config.with_path("/commands/update/status");

        let response = get_client().patch(url).json(&request).send().await?;

        let bind = |_: reqwest::Response| -> BoxFuture<'static, ApiResult<()>> {
            Box::pin(async move { Ok(()) })
        };

        handle_response(response, bind).await
    }

    #[cfg(test)]
    mod test {
        use crate::{
            api::models::update_command_status::UpdateCommandStatusRequest,
            models::{
                db::{
                    commands::{Command, CommandStatus},
                    common::{HasId, Id},
                },
                HandlerError,
            },
            test_commons::{before_each, get_404_json_string, get_500_json_string, setup_server},
        };

        fn get_json_payload(command_id: Id) -> (UpdateCommandStatusRequest, String) {
            let data = UpdateCommandStatusRequest::new(command_id);
            let data_string = serde_json::to_string(&data).unwrap();

            (data, data_string)
        }

        #[tokio::test]
        async fn test_update_command() {
            before_each();

            let command = Command::default();
            let (_, json) = get_json_payload(command.get_id().clone());
            let (mut server, config) = setup_server();

            let mock = server
                .mock("PATCH", "/commands/update/status")
                .with_status(200)
                .with_body(json)
                .create();

            let new_status = CommandStatus::Terminated;
            let result = super::update_command_status(&command, new_status, &config).await;

            assert!(result.is_ok());
            result.unwrap();
            mock.assert();
        }

        #[tokio::test]
        async fn test_update_commands_404_fail() {
            before_each();

            let command = Command::default();
            let (mut server, config) = setup_server();

            let mock = server
                .mock("PATCH", "/commands/update/status")
                .with_status(404)
                .with_body(get_404_json_string())
                .create();

            let new_status = CommandStatus::Terminated;
            let result = super::update_command_status(&command, new_status, &config).await;

            assert!(result.is_err());
            assert!(matches!(result.err().unwrap(), HandlerError::NotFound));
            mock.assert();
        }

        #[tokio::test]
        async fn test_update_commands_500_fail() {
            before_each();

            let command = Command::default();
            let (mut server, config) = setup_server();

            let mock = server
                .mock("PATCH", "/commands/update/status")
                .with_status(500)
                .with_body(get_500_json_string())
                .create();

            let new_status = CommandStatus::Terminated;
            let result = super::update_command_status(&command, new_status, &config).await;

            assert!(result.is_err());
            assert!(matches!(result.err().unwrap(), HandlerError::ServerError));
            mock.assert();
        }
    }
}

pub mod fetch_commands {
    use futures::future::BoxFuture;
    use log::info;

    use crate::api::models::fetch_commands::FetchRecentCommandResponse;
    use crate::api::requests::{get_client, handle_response, ApiResult};
    use crate::models::db::common::Id;

    use super::ApiConfig;

    pub async fn fetch_commands(
        device_id: Id,
        config: &ApiConfig,
    ) -> ApiResult<Option<FetchRecentCommandResponse>> {
        let url = config.with_path("/commands/recent");

        let response = get_client()
            .get(url)
            .query(&[("device_id", device_id)])
            .send()
            .await?;

        let status = response.status();
        info!("Response status for fetch commands: {}", status);

        let bind = |response: reqwest::Response| -> BoxFuture<'static, ApiResult<Option<FetchRecentCommandResponse>>> {
            Box::pin(async move { Ok(response.json().await?) })
        };
        handle_response(response, bind).await
    }

    #[cfg(test)]
    mod test {
        use crate::{
            api::models::fetch_commands::FetchRecentCommandResponse,
            models::{db::commands::Command, HandlerError},
            test_commons::{before_each, get_404_json_string, get_500_json_string, setup_server},
        };

        fn get_json_payload() -> (FetchRecentCommandResponse, String) {
            let data = FetchRecentCommandResponse::new(Command::default());
            let data_string = serde_json::to_string(&data).unwrap();

            (data, data_string)
        }

        #[tokio::test]
        async fn test_fetch_commands() {
            before_each();

            let (data, json) = get_json_payload();
            let device_id = data.command.device_id;
            let (mut server, config) = setup_server();

            let mock = server
                .mock(
                    "GET",
                    format!("/commands/recent?device_id={}", &device_id).as_str(),
                )
                .with_status(200)
                .with_body(json)
                .create();

            let result = super::fetch_commands(device_id.to_string(), &config).await;

            assert!(result.is_ok());
            let response = result.unwrap();
            assert!(response.is_some());
            assert_eq!(response.unwrap().command.device_id, device_id);
            mock.assert();
        }

        #[tokio::test]
        async fn test_fetch_commands_404_fail() {
            before_each();

            let (data, _) = get_json_payload();
            let device_id = data.command.device_id;
            let (mut server, config) = setup_server();

            let mock = server
                .mock(
                    "GET",
                    format!("/commands/recent?device_id={}", &device_id).as_str(),
                )
                .with_status(404)
                .with_body(get_404_json_string())
                .create();

            let result = super::fetch_commands(device_id.to_string(), &config).await;

            assert!(result.is_err());
            assert!(matches!(result.err().unwrap(), HandlerError::NotFound));
            mock.assert();
        }

        #[tokio::test]
        async fn test_fetch_commands_500_fail() {
            before_each();

            let (data, _) = get_json_payload();
            let device_id = data.command.device_id;
            let (mut server, config) = setup_server();

            let mock = server
                .mock(
                    "GET",
                    format!("/commands/recent?device_id={}", &device_id).as_str(),
                )
                .with_status(500)
                .with_body(get_500_json_string())
                .create();

            let result = super::fetch_commands(device_id.to_string(), &config).await;

            assert!(result.is_err());
            assert!(matches!(result.err().unwrap(), HandlerError::ServerError));
            mock.assert();
        }
    }
}
