use futures::future::BoxFuture;

use crate::api::models::register_device::{RegisterDeviceRequest, RegisterDeviceResponse};
use crate::api::requests::{get_client, handle_response, ApiResult};
use crate::models::db::common::Id;

use super::ApiConfig;

pub async fn register_device(
    user_id: &Id,
    user_secret: &str,
    device_name: String,
    config: &ApiConfig,
) -> ApiResult<RegisterDeviceResponse> {
    let request = RegisterDeviceRequest {
        user_id: user_id.clone(),
        issuer_id: user_id.clone(),
        device_name,
        user_secret: user_secret.to_string(),
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
        api::models::register_device::{RegisterDeviceRequest, RegisterDeviceResponse},
        models::{db::common::Id, HandlerError},
        test_commons::{before_each, get_404_json_string, get_500_json_string, setup_server},
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
        let result = super::register_device(&input.user_id,&input.user_secret, input.device_name, &config).await;

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
        let result = super::register_device(&input.user_id, &input.user_secret, input.device_name, &config).await;

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
        let result = super::register_device(&input.user_id, &input.user_secret, input.device_name, &config).await;

        assert!(result.is_err());
        assert!(matches!(result.err().unwrap(), HandlerError::ServerError));
        mock.assert();
    }
}
