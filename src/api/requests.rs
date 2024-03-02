use crate::models::HandlerError;
pub type ApiResult<T> = Result<T, HandlerError>;

fn get_port_string_if_any() -> String {
    "5001".to_string()
}

fn get_host() -> String {
    let port = get_port_string_if_any();
    format!("http://localhost:{}", port)
}

fn get_client() -> reqwest::Client {
    reqwest::Client::new()
}

pub mod register_device {
    use crate::api::models::register_device::{RegisterDeviceRequest, RegisterDeviceResponse};
    use crate::api::requests::{get_client, get_host, ApiResult};
    use crate::models::db::common::Id;

    fn get_url() -> String {
        let host = get_host();
        format!("{}/devices/register", host)
    }

    pub async fn register_device(
        user_id: &Id,
        device_name: String,
    ) -> ApiResult<RegisterDeviceResponse> {
        let request = RegisterDeviceRequest {
            user_id: user_id.clone(),
            device_name,
            issuer_id: user_id.clone(),
        };

        let url = get_url();

        let response = get_client().post(url).json(&request).send().await?;

        let status = response.status();
        println!("Response status: {}", status);

        let json = response.json().await?;
        Ok(json)
    }
}

pub mod update_command_status {
    use crate::api::models::update_command_status::UpdateCommandStatusRequest;
    use crate::api::requests::{get_client, get_host, ApiResult};
    use crate::models::db::commands::{Command, CommandStatus};
    use crate::models::db::common::HasId;

    fn get_url() -> String {
        let host = get_host();
        format!("{}/commands/update/status", host)
    }

    pub async fn update_command_status(
        command: &Command,
        new_status: CommandStatus,
    ) -> ApiResult<()> {
        let request = UpdateCommandStatusRequest {
            command_id: command.get_id().clone(),
            status: new_status,
        };

        let url = get_url();

        let response = get_client().patch(url).json(&request).send().await?;

        let status = response.status();
        println!("Response status: {}", status);

        let text = response.text().await?;
        println!("Response text: {}", text);

        Ok(())
    }
}

pub mod fetch_commands {
    use log::{info, warn};

    use crate::api::models::fetch_commands::FetchRecentCommandResponse;
    use crate::api::requests::{get_client, get_host, ApiResult};
    use crate::models::db::common::Id;

    fn get_url() -> String {
        let host = get_host();
        format!("{}/commands/recent", host)
    }

    pub async fn fetch_commands(device_id: Id) -> ApiResult<Option<FetchRecentCommandResponse>> {
        let url = get_url();

        let response = get_client()
            .get(url)
            .query(&[("device_id", device_id)])
            .send()
            .await?;

        let status = response.status();
        info!("Response status for fetch commands: {}", status);

        if status != 200 {
            warn!("No commands found: {}", response.text().await?);
            return Ok(None);
        }

        let json = response.json().await?;
        Ok(Some(json))
    }

    #[cfg(test)]
    mod test {
        use httpmock::Method::GET;
        use httpmock::MockServer;

        use crate::models::db::common::Id;

        #[tokio::test]
        async fn fetch_success() {
            // Start a mock server
            let server = MockServer::start();

            // Create a mock on the server
            let mock = server.mock(|when, then| {
                when.method(GET).path("/commands/recent");
                then.status(200);
            });

            // Call the function under test
            let result = super::fetch_commands(Id::new()).await;

            // Assert that the mock was called and the function returned the expected result
            mock.assert();
            assert!(result.is_ok());
            let response = result.unwrap();
            assert!(response.is_some());
        }
    }
}
