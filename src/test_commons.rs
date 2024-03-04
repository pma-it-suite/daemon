use std::sync::Mutex;

use crate::api::requests::get_port_string_if_any;
use lazy_static::lazy_static;
use mockito;

// wrap with mutex
// static mut SETUP_DONE: bool = false;
lazy_static! {
    static ref SETUP_DONE: Mutex<bool> = Mutex::new(false);
}

fn once() {
    let mut setup_done = SETUP_DONE.lock().unwrap();
    if *setup_done {
        return;
    }
    std::env::set_var("RUST_LOG", "info");
    simple_logger::SimpleLogger::new().env().init().unwrap();
    *setup_done = true;
}

pub fn before_each() {
    once();
}

pub fn setup_server() -> mockito::Server {
    let opts = mockito::ServerOpts {
        host: "127.0.0.1",
        port: get_port_string_if_any().parse::<u16>().unwrap(),
        ..Default::default()
    };
    mockito::Server::new_with_opts(opts)
}
