pub mod fetch_commands;
pub mod register_device;
pub mod update_command_status;

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
        let (host, port) = match get_host_string_from_localstore_or_env() {
            Ok(val) => {
                if val.is_empty() {
                    ("http://127.0.0.1".to_string(), Some(5001))
                } else {
                    (val, None)
                }
            }
            Err(e) => {
                error!("error getting host string: {}", e);
                ("http://127.0.0.1".to_string(), Some(5001))
            }
        };

        ApiConfig { host, port }
    }
}

fn get_host_string_from_localstore_or_env() -> Result<String, HandlerError> {
    let env_string = match std::env::var("ITX_API_HOST") {
        Ok(val) => Ok(val),
        Err(_) => Err(HandlerError::ApiError),
    };

    if env_string.is_ok() {
        env_string
    } else {
        let key = "api_host";
        match crate::localstore::query_data(key) {
            Ok(Some(val)) => Ok(val),
            _ => Err(HandlerError::ApiError),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::api::requests::ApiConfig;

    #[test]
    fn test_api_config_with_path() {
        let host = "testhost".to_string();
        let port = Some(5001);
        let config = ApiConfig::new(host, port);

        let path = "/testpath";
        let result = config.with_path(path);

        let expected = "testhost:5001/testpath";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_api_config_with_path_no_port() {
        let host = "testhost".to_string();
        let port = None;
        let config = ApiConfig::new(host, port);

        let path = "/testpath";
        let result = config.with_path(path);

        let expected = "testhost/testpath";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_default_api_config_with_path() {
        let config = ApiConfig::default();

        let path = "/testpath";
        let _ = config.with_path(path);
    }
}

async fn handle_response<T>(
    response: reqwest::Response,
    on_ok: impl Fn(reqwest::Response) -> BoxFuture<'static, Result<T, HandlerError>>,
) -> Result<T, HandlerError> {
    let status = response.status();
    if let StatusCode::OK | StatusCode::CREATED | StatusCode::NO_CONTENT = status {
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
