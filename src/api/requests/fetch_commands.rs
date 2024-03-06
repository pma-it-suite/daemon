
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
