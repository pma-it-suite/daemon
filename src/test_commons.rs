use std::sync::Mutex;

use crate::api::requests::ApiConfig;
use lazy_static::lazy_static;
use mockito;

lazy_static! {
    static ref SETUP_DONE: Mutex<bool> = Mutex::new(false);
}

fn once() {
    let mut setup_done = SETUP_DONE.lock().unwrap();
    if *setup_done {
        return;
    }
    std::env::set_var("RUST_LOG", "debug");
    simple_logger::SimpleLogger::new().env().init().unwrap();
    *setup_done = true;
}

pub fn before_each() {
    once();
}

pub fn get_api_config_with_port(port: u16) -> ApiConfig {
    ApiConfig::new("http://127.0.0.1".to_string(), Some(port))
}

pub fn setup_server_with_default() -> mockito::Server {
    let default_opts = ApiConfig::default();
    let opts = mockito::ServerOpts {
        host: default_opts.host.as_str(),
        port: default_opts.port.unwrap(),
        ..Default::default()
    };
    mockito::Server::new_with_opts(opts)
}

pub fn setup_server() -> (mockito::Server, ApiConfig) {
    let opts = mockito::ServerOpts {
        host: "127.0.0.1",
        ..Default::default()
    };
    let server = mockito::Server::new_with_opts(opts);

    let port = server.socket_address().port();

    (server, get_api_config_with_port(port))
}

pub fn get_404_json_string() -> String {
    r#"{"error": "not found"}"#.to_string()
}

pub fn get_500_json_string() -> String {
    "server error".to_string()
}
