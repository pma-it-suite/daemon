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
