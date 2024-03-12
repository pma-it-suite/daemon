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
    "testnewdevicefelipe".to_string()
}

pub fn get_user_id() -> Result<Id, HandlerError> {
    // prelim check for data
    let user_id_key = "user_id";
    let user_id = query_data(user_id_key)?.expect("user id to be set");
    info!("user_id retrieved from store is {}", &user_id);
    Ok(user_id)
}

pub fn get_user_secret() -> Result<String, HandlerError> {
    // prelim check for data
    let user_secret_key = "user_secret";
    let user_secret = query_data(user_secret_key)?.expect("user secret to be set");
    info!("user_secret retrieved from store is {}", &user_secret);

    Ok(user_secret)
}

async fn get_device_id_inner(
    user_id: &Id,
    user_secret: &str,
    config: ApiConfig,
) -> Result<Id, HandlerError> {
    // get device id or register it if not set
    let device_id_key = "device_id";
    let device_id_resp = query_data(device_id_key);
    let device_id = if device_id_resp.is_err() {
        let received_id = register_device_inner(user_id, user_secret, config).await?;
        info!("received device id from call and storing: {}", &received_id);
        write_single(&received_id, device_id_key)?;
        info!("stored device id: {}", &received_id);
        received_id
    } else {
        device_id_resp?.unwrap()
    };
    info!("device id retrieved from store is {}", device_id);

    Ok(device_id)
}

pub async fn get_device_id(user_id: &Id, user_secret: &str) -> Result<Id, HandlerError> {
    get_device_id_inner(user_id, user_secret, ApiConfig::default()).await
}

async fn register_device_inner(
    user_id: &Id,
    user_secret: &str,
    config: ApiConfig,
) -> Result<Id, HandlerError> {
    let device_name = get_device_name();
    info!("registering device with name: {}", device_name);
    Ok(
        api::requests::register_device::register_device(user_id, user_secret, device_name, &config)
            .await?
            .device_id,
    )
}

#[cfg(test)]
mod test {
    #![allow(clippy::await_holding_lock)]
    use std::sync::Mutex;

    use crate::{
        api::models::register_device::{RegisterDeviceRequest, RegisterDeviceResponse},
        localstore::{get_handle, write_single},
        models::db::common::Id,
        test_commons::{before_each_fs, setup_server},
    };

    use lazy_static::lazy_static;

    fn get_json_payload(device_id: Id) -> (RegisterDeviceResponse, String) {
        let data = RegisterDeviceResponse::new(device_id);
        let data_string = serde_json::to_string(&data).unwrap();

        (data, data_string)
    }

    lazy_static! {
        static ref LOCK: Mutex<()> = Mutex::new(());
    }

    #[test]
    fn test_get_user_id_ok() {
        let _tmp = LOCK.lock().unwrap();
        before_each_fs();

        let _ = get_handle().unwrap();

        let result = super::get_user_id();
        assert!(result.is_ok());
        assert!(result.unwrap() != *"");
    }

    #[tokio::test]
    async fn test_register_device() {
        let _tmp = LOCK.lock().unwrap();
        before_each_fs();

        let _ = get_handle().unwrap();
        let device_id = "testdeviceid".to_string();
        let (data, json) = get_json_payload(device_id);
        let (mut server, config) = setup_server();

        let mock = server
            .mock("POST", "/devices/register")
            .with_status(200)
            .with_body(json)
            .create();

        let input = RegisterDeviceRequest::default();
        let result = super::register_device_inner(&input.user_id, &input.user_secret, config).await;
        dbg!(&result);

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response, data.device_id);
        mock.assert();
    }

    #[tokio::test]
    async fn test_get_device_id_ok_when_no_existing() {
        let _tmp = LOCK.lock().unwrap();
        before_each_fs();

        let _ = get_handle().unwrap();
        let device_id = "testdeviceid".to_string();
        let (data, json) = get_json_payload(device_id);
        let (mut server, config) = setup_server();

        let mock = server
            .mock("POST", "/devices/register")
            .with_status(200)
            .with_body(json)
            .create();

        let user_id = "testid".to_string();
        let user_secret = "secret".to_string();
        let result = super::get_device_id_inner(&user_id, &user_secret, config).await;

        assert!(result.is_ok());
        assert!(result.unwrap() == data.device_id);
        mock.assert();
    }

    #[tokio::test]
    async fn test_get_device_id_ok_when_existing() {
        let _tmp = LOCK.lock().unwrap();
        before_each_fs();

        let _ = get_handle().unwrap();
        let (key, device_id) = ("device_id".to_string(), "testdeviceid".to_string());
        let response = write_single(&device_id, &key);
        assert!(response.is_ok());

        let user_id = "testid".to_string();
        let user_secret = "testsecret".to_string();
        let result = super::get_device_id(&user_id, &user_secret).await;

        assert!(result.is_ok());
        assert!(result.unwrap() == device_id);
    }
}
