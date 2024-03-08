use log::info;

use crate::{
    api::{self, requests::ApiConfig},
    localstore::{query_data, write_single},
    models::{db::common::Id, HandlerError},
};

/**
 * main (pre-registered) run loop:
 * 1. register device with server
 * 2. test connection to server
 */
pub fn get_device_name() -> String {
    "testdevicefelipearce".to_string()
}

pub fn get_user_id() -> Result<Id, HandlerError> {
    // prelim check for data
    let user_id_key = "user_id";
    let user_id = query_data(user_id_key)?.expect("user id to be set");
    info!("user_id retrieved from store is {}", &user_id);

    Ok(user_id)
}

pub async fn get_device_id(user_id: &Id) -> Result<Id, HandlerError> {
    // get device id or register it if not set
    let device_id_key = "device_id";
    let device_id_resp = query_data(device_id_key);
    let device_id: Id;
    if device_id_resp.is_err() {
        device_id = register_device(user_id).await?;
        write_single(&device_id, device_id_key)?;
    } else {
        device_id = device_id_resp?.unwrap();
    }
    info!("device id retrieved from store is {}", device_id);

    Ok(device_id)
}

pub async fn register_device(user_id: &Id) -> Result<Id, HandlerError> {
    let device_name = get_device_name();
    info!("registering device with name: {}", device_name);

    Ok(
        api::requests::register_device::register_device(
            user_id,
            device_name,
            &ApiConfig::default(),
        )
        .await?
        .device_id,
    )
}

#[cfg(test)]
mod test {
    use crate::{
        api::models::register_device::{RegisterDeviceRequest, RegisterDeviceResponse},
        models::{db::common::Id, HandlerError},
        test_commons::{
            before_each, get_404_json_string, get_500_json_string, setup_server,
            setup_server_with_default,
        },
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
        let mut server = setup_server_with_default();

        let mock = server
            .mock("POST", "/devices/register")
            .with_status(200)
            .with_body(json)
            .create();

        let input = RegisterDeviceRequest::default();
        let result = super::register_device(&input.user_id).await;
        dbg!(&result);

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response, data.device_id);
        mock.assert();
    }
}
