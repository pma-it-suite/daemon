#![feature(async_closure)]
#![feature(never_type)]

use log::{error, LevelFilter};
use log4rs::{
    self,
    append::rolling_file::{
        policy::compound::{
            roll::fixed_window::FixedWindowRoller, trigger::size::SizeTrigger, CompoundPolicy,
        },
        RollingFileAppender,
    },
    config::{Appender, Root},
    encode::pattern::PatternEncoder,
    filter::threshold::ThresholdFilter,
    Config,
};
use main_event_loop::run_main_event_loop;
use models::HandlerError;
use pre_event_loop::{get_device_id, get_user_id, get_user_secret};

use crate::main_event_loop::{sleep_in_seconds, SLEEP_LONG};
mod models;

#[tokio::main]
async fn main() -> ! {
    std::env::set_var("RUST_LOG", "info");

    let log_result = setup_logger();

    if log_result.is_err() {
        simple_logger::SimpleLogger::new()
            .env()
            .with_utc_timestamps()
            .init()
            .unwrap();
    }

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

fn setup_logger() -> Result<(), HandlerError> {
    let window_size = 3; // log0, log1, log2
    let fixed_window_roller = FixedWindowRoller::builder().build("log{}", window_size)?;

    let size_limit = 5 * 1024; // 5KB as max log file size to roll
    let size_trigger = SizeTrigger::new(size_limit);
    let compound_policy =
        CompoundPolicy::new(Box::new(size_trigger), Box::new(fixed_window_roller));
    let config = Config::builder()
        .appender(
            Appender::builder()
                .filter(Box::new(ThresholdFilter::new(LevelFilter::Debug)))
                .build(
                    "logfile",
                    Box::new(
                        RollingFileAppender::builder()
                            .encoder(Box::new(PatternEncoder::new("{d} {l}::{m}{n}")))
                            .build("logfile", Box::new(compound_policy))?,
                    ),
                ),
        )
        .build(
            Root::builder()
                .appender("logfile")
                .build(LevelFilter::Debug),
        )?;
    log4rs::init_config(config)?;

    Ok(())
}
