#![feature(async_closure)]
#![feature(never_type)]

use log::error;
use main_event_loop::run_main_event_loop;
use pre_event_loop::{get_device_id, get_user_id, get_user_secret};

use crate::main_event_loop::{sleep_in_seconds, SLEEP_LONG};
mod models;

#[tokio::main]
async fn main() -> ! {
    std::env::set_var("RUST_LOG", "info");
    simple_logger::SimpleLogger::new().env().init().unwrap();

    // pre event loop
    let user_id;
    loop {
        let resp = get_user_id();
        match resp {
            Ok(id) => {
                user_id = id;
                break;
            }
            Err(e) => {
                error!("error getting user id: {:#?}", e);
                sleep_in_seconds(SLEEP_LONG);
            }
        }
    }

    let user_secret;
    loop {
        let resp = get_user_secret();
        match resp {
            Ok(secret) => {
                user_secret = secret;
                break;
            }
            Err(e) => {
                error!("error getting user secret: {:#?}", e);
                sleep_in_seconds(SLEEP_LONG);
            }
        }
    }

    let device_id;
    loop {
        let resp = get_device_id(&user_id, &user_secret).await;
        match resp {
            Ok(id) => {
                device_id = id;
                break;
            }
            Err(e) => {
                error!("error getting device id: {:#?}", e);
                sleep_in_seconds(SLEEP_LONG);
            }
        }
    }

    // run main event loop
    run_main_event_loop(&device_id, &user_id).await
}

pub mod api;
pub mod executor;
pub mod localstore;
pub mod main_event_loop;
pub mod pre_event_loop;

#[cfg(test)]
pub mod test_commons;
