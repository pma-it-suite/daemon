#![feature(async_closure)]

use main_event_loop::run_main_event_loop;
use models::HandlerError;
use pre_event_loop::{get_device_id, get_user_id};
mod models;

#[tokio::main]
async fn main() -> Result<(), HandlerError> {
    std::env::set_var("RUST_LOG", "info");
    simple_logger::SimpleLogger::new().env().init().unwrap();

    // pre event loop
    let user_id = get_user_id()?;
    let device_id = get_device_id(&user_id).await?;

    // run main event loop
    run_main_event_loop(&device_id, &user_id).await?;

    Ok(())
}

pub mod api;
pub mod localstore;
pub mod main_event_loop;
pub mod pre_event_loop;
pub mod executor;


#[cfg(test)]
pub mod test_commons;
