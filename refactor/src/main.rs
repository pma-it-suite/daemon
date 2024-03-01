
use models::HandlerError;
use pre_event_loop::{get_device_id, get_user_id};
mod models;

#[tokio::main]
async fn main() -> Result<(), HandlerError> {
    std::env::set_var("RUST_LOG", "info");
    simple_logger::SimpleLogger::new().env().init().unwrap();

    // pre event loop
    let user_id = get_user_id()?;
    let _device_id = get_device_id(&user_id).await?;

    // run main event loop

    Ok(())
}

pub mod pre_event_loop;
pub mod main_event_loop;
pub mod api;
pub mod localstore;
