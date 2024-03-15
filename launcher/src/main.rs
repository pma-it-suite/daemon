#![feature(async_closure)]
#![feature(never_type)]
#![feature(build_hasher_simple_hash_one)]

use std::fs::{self};
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::path::PathBuf;

use localstore::LocalStore;
use log::{debug, error, info};
use models::{AppConfig, GetBinPath, HandlerResult, LauncherConfig, SemVer};
use requests::{
    upstream_requests::{fetch_bin, fetch_version, BinData},
    ApiConfig,
};

use crate::models::HandlerError;

#[tokio::main]
async fn main() -> ! {
    /*
     * 1. make sure launcher config is good on local
     *      - NOTE: for now, hardcoded default. Eventually populate from initial distribution pkg
     *
     * 2. do health check on upstream until healthy
     *      - if bad resp -> sleep long and loop
     *
     * 3. check if app has been installed in local machine
     *
     * 4. pull semver from upstream
     *
     * 5.a. if app NOT installed
     *      - pull bin from upstream
     *      - install bin on local
     *      - setup first time configs for app
     *      - update configs for launcher
     *
     * 5.b. if app IS installed
     *      - check semver of app config
     *      - if local semver < upstream semver
     *          - pull bin from upstream
     *          - nuke bin on local
     *          - install bin on local
     *          - update configs for app
     *          - update configs for launcher
     *      - else do nothing
     *
     * 6. run app with launcherd
     * 7. monitor and set schedule to start from step (1) every N hours/minutes/days
     */
    std::env::set_var("RUST_LOG", "info");
    simple_logger::SimpleLogger::new().env().init().unwrap();

    info!("Launcher starting...");

    let launcher_store = match does_local_launcher_config_exist() {
        true => {
            info!("Launcher configuration found.");

            LocalStore::new(PathBuf::from(get_launcher_store_file_name()))
                .expect("Failed to create LocalStore.")
        }
        false => {
            info!("No launcher configuration found, creating...");

            let init_config = LauncherConfig {
        app_path: get_path(),
        // Additional logging for configuration load
        bin_name: "itx".to_string(),
        app_version: SemVer::new(0, 0, 0),
        launcher_version: SemVer::new(0, 0, 1),
        user_id: "9c66d842-cab9-4bff-93be-b05388f652e7".to_string(),
        user_secret: "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzZWNyZXQiOiJmZDI5MTRlYy0wMTBhLTRkNDYtYjk1YS01MzdhMzRmYWQ3MjIiLCJ1c2VyX2lkIjoiOWM2NmQ4NDItY2FiOS00YmZmLTkzYmUtYjA1Mzg4ZjY1MmU3IiwiZXhwIjoxNzEwNDU1OTg2fQ.II_lTbMcMp4-dywN4QAorqdJBZobM8cyC-KTgp96GeY".to_string(),
        has_app_been_installed: false,
    };

            debug!("Launcher initial configuration loaded.");

            match LocalStore::from(PathBuf::from(get_launcher_store_file_name()), &init_config) {
                Ok(store) => store,
                Err(e) => {
                    error!("Failed to create LocalStore from launcherdata.json: {}", e);
                    panic!("Launcher initialization failure.");
                }
            }
        }
    };

    let mut config: LauncherConfig = match launcher_store.get_all_data() {
        Ok(cfg) => cfg,
        Err(e) => {
            error!("Failed to get launcher configuration: {}", e);
            panic!("Launcher configuration retrieval failure.");
        }
    };

    info!("Pinging upstream server...");
    assert!(requests::upstream_requests::ping(&ApiConfig::default())
        .await
        .expect("Failed to ping upstream server."));

    info!("Fetching upstream version...");
    let upstream_version = get_upstream_app_version()
        .await
        .expect("Failed to fetch upstream version.");

    let app_store = LocalStore::new(config.app_path.join("appdata.json"))
        .expect("Failed to create app LocalStore.");

    info!("Checking app configuration...");
    let app_config_result = get_config_from_app_local(&app_store);

    if app_config_result.is_ok() {
        info!("App configuration found.");
    } else {
        info!("No app configuration found, setting launcher config to not installed...");
        config.has_app_been_installed = false;
    }

    debug!("Processing based on app installation status...");
    match config.has_app_been_installed {
        false => {
            info!("App not installed, pulling from upstream...");
            pull_from_upstream_and_install_binary_to_local(&config)
                .await
                .expect("Failed to install binary from upstream.");

            debug!("Saving app configuration...");
            let app_config = AppConfig {
                device_id: "".to_string(),
                bin_name: config.bin_name.clone(),
                app_path: config.app_path.clone(),
                version: upstream_version,
                user_id: config.user_id.clone(),
                user_secret: config.user_secret.clone(),
            };
            save_app_config_to_local(&app_store, &app_config)
                .expect("Failed to save app configuration.");

            config.has_app_been_installed = true;
            config.app_version = upstream_version;
            save_launcher_config_to_local(&launcher_store, &config)
                .expect("Failed to save launcher configuration.");
        }
        true => {
            let mut app_config = app_config_result.expect("Failed to get app configuration.");

            if app_config.version < upstream_version {
                info!("New version available, updating app...");
                pull_from_upstream_and_install_binary_to_local(&config)
                    .await
                    .expect("Failed to install binary from upstream.");
                app_config.version = upstream_version;

                save_app_config_to_local(&app_store, &app_config)
                    .expect("Failed to save updated app configuration.");
                config.app_version = upstream_version;
                save_launcher_config_to_local(&launcher_store, &config)
                    .expect("Failed to save updated launcher configuration.");
            } else {
                info!("App is up to date, continuing to launch...");
            }
        }
    }

    info!("Launching app...");
    let app_config =
        get_config_from_app_local(&app_store).expect("Failed to get app configuration for launch.");
    launch_app_with_launcherd(&app_config).expect("Failed to launch app.");

    panic!();
}

fn get_launcher_store_file_name() -> String {
    "launcherdata.json".to_string()
}

fn does_local_launcher_config_exist() -> bool {
    let path = PathBuf::from(get_launcher_store_file_name());
    path.exists()
}

pub fn launch_app_with_launcherd(config: &AppConfig) -> HandlerResult<()> {
    let file_path = &config.get_bin_path();
    match set_execute_permission(file_path) {
        Ok(_) => {
            info!(
                "Execute permissions set for file: {}",
                file_path.to_str().unwrap()
            );
        }
        Err(e) => {
            error!("Failed to set execute permissions: {}", e);
            return Err(HandlerError::from(e));
        }
    }

    info!("launching app with file: {}", file_path.to_str().unwrap());
    let mut cmd = std::process::Command::new(file_path);
    cmd.spawn()?;
    Ok(())
}

fn set_execute_permission(file_path: &Path) -> std::io::Result<()> {
    // let parent = file_path.parent().unwrap();
    let parent = file_path;
    let metadata = fs::metadata(parent)?;
    let mut permissions = metadata.permissions();

    // This adds execute permissions for the owner, group, and others
    permissions.set_mode(0o755); // Read & execute for everyone, write for owner

    fs::set_permissions(parent, permissions)?;

    Ok(())
}

pub async fn get_upstream_app_version() -> HandlerResult<SemVer> {
    let config = ApiConfig::default();
    fetch_version(&config).await
}

pub async fn get_binary_from_upstream() -> HandlerResult<BinData> {
    let config = ApiConfig::default();
    fetch_bin(&config).await
}

pub async fn pull_from_upstream_and_install_binary_to_local(
    config: &LauncherConfig,
) -> HandlerResult<()> {
    create_app_dir_if_none_exists(config)?;
    let file_name = &config.get_bin_path();
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(file_name)?;
    let mut content = get_binary_from_upstream().await?;
    std::io::copy(&mut content, &mut file)?;
    Ok(())
}

pub fn get_config_from_launcher_local(store: &LocalStore) -> HandlerResult<AppConfig> {
    store.get_all_data()
}

pub fn get_config_from_app_local(store: &LocalStore) -> HandlerResult<AppConfig> {
    store.get_all_data()
}

pub fn save_app_config_to_local(store: &LocalStore, config: &AppConfig) -> HandlerResult<()> {
    store.overwrite_current_data(config)
}

pub fn save_launcher_config_to_local(
    store: &LocalStore,
    config: &LauncherConfig,
) -> HandlerResult<()> {
    store.overwrite_current_data(config)
}

pub fn has_been_installed(config: &LauncherConfig) -> bool {
    let base_path = config.app_path.parent().unwrap();
    base_path.exists()
}

pub fn create_app_dir_if_none_exists(config: &LauncherConfig) -> HandlerResult<()> {
    let base_path = config.app_path.parent().unwrap();
    if !base_path.exists() {
        std::fs::create_dir_all(base_path)?;
    }
    // also create file if none
    if !config.get_bin_path().exists() {
        std::fs::File::create(config.get_bin_path())?;
    }
    Ok(())
}

fn get_path() -> PathBuf {
    // check the ITX_PATH env var and set to that, else set to default
    match std::env::var("ITX_PATH") {
        Ok(val) => PathBuf::from(val),
        Err(_) => get_default_path(),
    }
}

fn get_default_path() -> PathBuf {
    PathBuf::from(shellexpand::tilde("~/.itx").into_owned())
}

pub mod models {

    use serde::{Deserialize, Serialize};
    use std::{cmp::Ordering, path::PathBuf};

    use thiserror::Error;

    pub type Id = String;

    pub type HandlerResult<T> = Result<T, HandlerError>;

    #[derive(Error, Debug)]
    pub enum HandlerError {
        #[error("io error")]
        IoError(#[from] std::io::Error),
        #[error("reqwest error")]
        ReqwestError(#[from] reqwest::Error),
        #[error("api client error")]
        ApiError,
        #[error("serde error")]
        SerError(#[from] serde_json::Error),
        #[error("unknown error")]
        Unknown,
        #[error("unknown error")]
        DecodingError(#[from] std::string::FromUtf8Error),
        #[error("not found error 404")]
        NotFound,
        #[error("cmd error")]
        CmdError(Id),
        #[error("parse cmd error")]
        ParseError(String),
        #[error("db error")]
        DbError,
        #[error("server error 500")]
        ServerError,
        #[error("input error 4XX")]
        InputError,
    }

    #[derive(Debug, PartialEq, Eq, Deserialize, Clone, Default, Serialize, Copy)]
    pub struct SemVer {
        pub major: u32,
        pub minor: u32,
        pub patch: u32,
    }

    impl PartialOrd for SemVer {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            Some(self.cmp(other))
        }
    }

    impl Ord for SemVer {
        fn cmp(&self, other: &Self) -> Ordering {
            let this: u32 = format!("{}{}{}", self.major, self.minor, self.patch)
                .parse()
                .unwrap();
            let other: u32 = format!("{}{}{}", other.major, other.minor, other.patch)
                .parse()
                .unwrap();
            this.cmp(&other)
        }
    }

    impl SemVer {
        pub fn new(major: u32, minor: u32, patch: u32) -> Self {
            SemVer {
                major,
                minor,
                patch,
            }
        }

        pub fn is_breaking_change(&self, other: &SemVer) -> bool {
            self.major != other.major
        }
    }

    #[derive(Debug, Serialize, PartialEq, Eq, Deserialize, Clone, Default)]
    pub struct AppConfig {
        pub app_path: PathBuf,
        pub bin_name: String,
        pub version: SemVer,
        pub user_id: String,
        pub user_secret: String,
        pub device_id: String,
    }

    #[derive(Debug, Serialize, PartialEq, Eq, Deserialize, Clone, Default)]
    pub struct LauncherConfig {
        pub bin_name: String,
        pub app_path: PathBuf,
        pub app_version: SemVer,
        pub launcher_version: SemVer,
        pub user_id: String,
        pub user_secret: String,
        pub has_app_been_installed: bool,
    }

    pub trait GetBinPath {
        fn get_bin_path(&self) -> PathBuf;
    }

    impl GetBinPath for AppConfig {
        fn get_bin_path(&self) -> PathBuf {
            self.app_path.join(&self.bin_name)
        }
    }

    impl GetBinPath for LauncherConfig {
        fn get_bin_path(&self) -> PathBuf {
            self.app_path.join(&self.bin_name)
        }
    }
}

pub mod localstore;

pub mod requests {
    use crate::models::HandlerError;
    use futures::future::BoxFuture;
    use log::{error, warn};
    use reqwest::StatusCode;
    pub type ApiResult<T> = Result<T, HandlerError>;

    fn get_client() -> reqwest::Client {
        reqwest::Client::new()
    }

    pub struct ApiConfig {
        pub host: String,
        pub port: Option<u16>,
    }

    impl ApiConfig {
        pub fn new(host: String, port: Option<u16>) -> Self {
            ApiConfig { host, port }
        }

        fn get_port_string_if_any(&self) -> String {
            match self.port {
                Some(val) => format!(":{}", &val),
                None => "".to_string(),
            }
        }

        pub fn with_path(&self, path: &str) -> String {
            let port_string = self.get_port_string_if_any();
            format!("{}{}{}", self.host, port_string, path)
        }
    }

    impl Default for ApiConfig {
        fn default() -> Self {
            ApiConfig {
                host: "https://blobperma.blob.core.windows.net/blob-bin".to_string(),
                port: None, // host: "https://api.itx-app.com".to_string(),
                            // port: None,
            }
        }
    }

    pub mod upstream_requests {
        use std::io::Cursor;

        use futures::future::BoxFuture;
        use log::info;
        use serde::{Deserialize, Serialize};

        pub type BinData = Cursor<bytes::Bytes>;

        use crate::{
            models::HandlerResult,
            requests::{get_client, handle_response, ApiResult},
            SemVer,
        };

        use super::ApiConfig;

        #[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
        struct Ping {
            ok: bool,
        }

        pub async fn fetch_version(config: &ApiConfig) -> HandlerResult<SemVer> {
            let url = config.with_path("/semver.json");
            let response = get_client().get(url).send().await?;

            let status = response.status();
            info!("Response status for fetch version: {}", status);

            let bind = |response: reqwest::Response| -> BoxFuture<'static, ApiResult<SemVer>> {
                Box::pin(async move { Ok(response.json().await?) })
            };
            handle_response(response, bind).await
        }

        pub async fn fetch_bin(config: &ApiConfig) -> HandlerResult<BinData> {
            let url = config.with_path("/bintest");
            let response = get_client().get(url).send().await?;

            let status = response.status();
            info!("Response status for fetch bin: {}", status);

            let bind = |response: reqwest::Response| -> BoxFuture<'static, ApiResult<BinData>> {
                Box::pin(async move { Ok(Cursor::new(response.bytes().await?)) })
            };
            handle_response(response, bind).await
        }

        pub async fn ping(config: &ApiConfig) -> HandlerResult<bool> {
            let url = config.with_path("/ping.json");
            let response = get_client().get(url).send().await?;

            let status = response.status();
            info!("Response status for health check: {}", status);

            let bind = |response: reqwest::Response| -> BoxFuture<'static, ApiResult<Ping>> {
                Box::pin(async move { Ok(response.json().await?) })
            };
            let response = handle_response(response, bind).await;

            match response {
                Ok(ping) => Ok(ping.ok),
                Err(_) => Ok(false),
            }
        }
    }

    #[cfg(test)]
    mod test {
        use crate::requests::ApiConfig;

        #[test]
        fn test_api_config_with_path() {
            let host = "testhost".to_string();
            let port = Some(5001);
            let config = ApiConfig::new(host, port);

            let path = "/testpath";
            let result = config.with_path(path);

            let expected = "testhost:5001/testpath";
            assert_eq!(result, expected);
        }

        #[test]
        fn test_api_config_with_path_no_port() {
            let host = "testhost".to_string();
            let port = None;
            let config = ApiConfig::new(host, port);

            let path = "/testpath";
            let result = config.with_path(path);

            let expected = "testhost/testpath";
            assert_eq!(result, expected);
        }

        #[test]
        fn test_default_api_config_with_path() {
            let config = ApiConfig::default();

            let path = "/testpath";
            let _ = config.with_path(path);
        }
    }

    async fn handle_response<T>(
        response: reqwest::Response,
        on_ok: impl Fn(reqwest::Response) -> BoxFuture<'static, Result<T, HandlerError>>,
    ) -> Result<T, HandlerError> {
        let status = response.status();
        if let StatusCode::OK | StatusCode::CREATED | StatusCode::NO_CONTENT = status {
            Ok(on_ok(response).await?)
        } else {
            let text = response.text().await?;
            match status {
                StatusCode::NOT_FOUND => {
                    warn!("No commands found: {}", &text);
                    Err(HandlerError::NotFound)
                }
                StatusCode::INTERNAL_SERVER_ERROR => {
                    error!("server error on fetch: {}", &text);
                    Err(HandlerError::ServerError)
                }
                StatusCode::BAD_REQUEST | StatusCode::UNPROCESSABLE_ENTITY => {
                    warn!("error in data passed in: {}", &text);
                    Err(HandlerError::InputError)
                }
                _ => {
                    warn!("unknown error code: {}, {}", &text, status);
                    Err(HandlerError::ApiError)
                }
            }
        }
    }
}
