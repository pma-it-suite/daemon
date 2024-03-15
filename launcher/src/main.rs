#![feature(async_closure)]
#![feature(never_type)]
#![feature(build_hasher_simple_hash_one)]

use service_manager::*;

use std::fs::{self};
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;
use std::{env, thread};

use localstore::LocalStore;
use log::{debug, error, info};
use models::{AppConfig, GetBinPath, HandlerResult, LauncherConfig, SemVer};
use requests::{
    upstream_requests::{fetch_bin, fetch_version, BinData},
    ApiConfig,
};

use crate::models::HandlerError;

const SLEEP_VERY_LONG: u64 = 60;
const SLEEP_EXTREMELY_LONG: u64 = 60 * 60 * 24; // 24 hours

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

    info!("Launcher starting...");

    // if arg passed in is "start, stop, restart" then do handle and panic early
    // else do normal flow
    loop {
        match simple_logger::SimpleLogger::new().env().init() {
            Ok(_) => {}
            Err(e) => {
                error!("Failed to setup logger: {}", e);
                sleep_in_seconds(SLEEP_VERY_LONG);
                continue;
            }
        }

        if does_have_args_and_handle_input_args() {
            panic!("with args - break loop");
        }

        let (launcher_store, mut config) = match get_launcher_config_and_store() {
            Ok((store, cfg)) => (store, cfg),
            Err(e) => {
                error!("Failed to get launcher configuration: {}", e);
                sleep_in_seconds(SLEEP_VERY_LONG);
                continue;
            }
        };

        let upstream_version = match ping_server_and_get_upstream_app_version().await {
            Ok(version) => version,
            Err(e) => {
                error!("Failed to get upstream app version: {}", e);
                sleep_in_seconds(SLEEP_VERY_LONG);
                continue;
            }
        };

        let (app_store, app_config_result) = match get_app_store_and_data(&config) {
            Ok((store, cfg)) => (store, cfg),
            Err(e) => {
                error!("Failed to get app configuration: {}", e);
                sleep_in_seconds(SLEEP_VERY_LONG);
                continue;
            }
        };

        if app_config_result.is_some() {
            info!("App configuration found.");
        } else {
            info!("No app configuration found, setting launcher config to not installed...");
            config.has_app_been_installed = false;
        }

        let mut needs_start = false;

        let binded_manager = match get_manager() {
            Ok(manager) => manager,
            Err(e) => {
                error!("Failed to get service manager: {}", e);
                sleep_in_seconds(SLEEP_VERY_LONG);
                continue;
            }
        };
        let manager = binded_manager.as_ref();

        debug!("Processing based on app installation status...");

        let needs_install = match config.has_app_been_installed {
            false => {
                match install_app_fresh(
                    &mut config,
                    manager,
                    &launcher_store,
                    &app_store,
                    upstream_version,
                )
                .await
                {
                    Ok(_) => {
                        info!("App installed successfully.");
                    }
                    Err(e) => {
                        error!("Failed to install app: {}", e);
                        continue;
                    }
                }

                needs_start = true;
                true
            }
            true => {
                let mut app_config = match app_config_result.is_some() {
                    true => app_config_result.unwrap(),
                    false => {
                        error!("App configuration not found.");
                        sleep_in_seconds(SLEEP_VERY_LONG);
                        continue;
                    }
                };

                if app_config.version < upstream_version {
                    match install_in_place(
                        &mut config,
                        manager,
                        &launcher_store,
                        &app_store,
                        upstream_version,
                        &mut app_config,
                    )
                    .await
                    {
                        Ok(_) => {
                            info!("App updated successfully.");
                        }
                        Err(e) => {
                            error!("Failed to update app: {}", e);
                            continue;
                        }
                    }
                    needs_start = true;
                } else {
                    info!("App is up to date. No launch required");
                    needs_start = false;
                }

                false
            }
        };

        info!("Launching app...");
        let app_config = match get_config_from_app_local(&app_store) {
            Ok(cfg) => cfg,
            Err(e) => {
                error!("Failed to get app configuration: {}", e);
                sleep_in_seconds(SLEEP_VERY_LONG);
                continue;
            }
        };

        if needs_install {
            match install_service(manager, &app_config) {
                Ok(_) => {
                    info!("Service installed successfully.");
                }
                Err(e) => {
                    error!("Failed to install service: {}", e);
                    sleep_in_seconds(SLEEP_VERY_LONG);
                }
            }
        }

        if needs_start {
            info!("Start requested, starting service...");
            match start_service(manager) {
                Ok(_) => {
                    info!("Service started successfully.");
                }
                Err(e) => {
                    error!("Failed to start service: {}", e);
                    sleep_in_seconds(SLEEP_VERY_LONG);
                }
            }
        } else {
            info!("No start requested, skipping service start.");
            sleep_in_seconds(SLEEP_EXTREMELY_LONG);
        }
    }
}

pub fn does_have_args_and_handle_input_args() -> bool {
    let args: Vec<String> = env::args().collect();
    if args.len() == 2 {
        let binded_manager = get_manager().expect("Failed to get service manager.");
        let manager = binded_manager.as_ref();
        match args[1].as_str() {
            "start" => {
                info!("Starting service...");
                start_service(manager).expect("Failed to start service.");
                true
            }
            "stop" => {
                info!("Stopping service...");
                stop_service(manager).expect("Failed to stop service.");
                true
            }
            "restart" => {
                info!("Restarting service...");
                stop_service(manager).expect("Failed to stop service.");
                start_service(manager).expect("Failed to start service.");
                true
            }
            _ => false,
        }
    } else {
        false
    }
}

pub fn get_launcher_config_and_store() -> HandlerResult<(LocalStore, LauncherConfig)> {
    let launcher_store = match does_local_launcher_config_exist() {
        true => {
            info!("Launcher configuration found.");

            LocalStore::new(PathBuf::from(get_launcher_store_file_name()))?
        }
        false => {
            info!("No launcher configuration found, creating...");
            let (user_id, user_secret) = get_secrets_from_env();

            let init_config = LauncherConfig {
                app_path: get_path(),
                // Additional logging for configuration load
                bin_name: "itx".to_string(),
                app_version: SemVer::new(0, 0, 0),
                launcher_version: SemVer::new(0, 0, 1),
                user_id,
                user_secret,
                has_app_been_installed: false,
            };

            debug!("Launcher initial configuration loaded.");

            match LocalStore::from(PathBuf::from(get_launcher_store_file_name()), &init_config) {
                Ok(store) => store,
                Err(e) => {
                    error!("Failed to create LocalStore from launcherdata.json: {}", e);
                    return Err(e);
                }
            }
        }
    };

    let config: LauncherConfig = match launcher_store.get_all_data() {
        Ok(cfg) => cfg,
        Err(e) => {
            error!("Failed to get launcher configuration: {}", e);
            return Err(e);
        }
    };
    Ok((launcher_store, config))
}

pub async fn ping_server_and_get_upstream_app_version() -> HandlerResult<SemVer> {
    info!("Pinging upstream server...");
    let resp = requests::upstream_requests::ping(&ApiConfig::default()).await;
    if let Ok(true) = resp {
        info!("Upstream server is healthy.");
    } else {
        error!("Upstream server is unhealthy.");
        return Err(HandlerError::Unknown);
    }

    info!("Fetching upstream version...");
    let upstream_version = get_upstream_app_version().await?;

    Ok(upstream_version)
}

pub fn get_app_store_and_data(
    config: &LauncherConfig,
) -> HandlerResult<(LocalStore, Option<AppConfig>)> {
    let app_store = LocalStore::new(config.app_path.join("appdata.json"))?;

    info!("Checking app configuration...");
    match get_config_from_app_local(&app_store) {
        Ok(cfg) => {
            info!("App configuration found.");
            Ok((app_store, Some(cfg)))
        }
        Err(e) => {
            error!("Failed to get app configuration: {}", e);
            Ok((app_store, None))
        }
    }
}

pub async fn install_app_fresh(
    config: &mut LauncherConfig,
    manager: &dyn ServiceManager,
    launcher_store: &LocalStore,
    app_store: &LocalStore,
    upstream_version: SemVer,
) -> HandlerResult<()> {
    info!("Attempting to stop and uninstall service as failsafe (ignoring failures)...");
    let _ = stop_service(manager);
    let _ = uninstall_service(manager);

    info!("App not installed, pulling from upstream...");
    pull_from_upstream_and_install_binary_to_local(config).await?;

    debug!("Saving app configuration...");
    let app_config = AppConfig {
        bin_name: config.bin_name.clone(),
        app_path: config.app_path.clone(),
        version: upstream_version,
        user_id: config.user_id.clone(),
        user_secret: config.user_secret.clone(),
    };
    save_app_config_to_local(app_store, &app_config)?;

    config.has_app_been_installed = true;
    config.app_version = upstream_version;
    save_launcher_config_to_local(launcher_store, config)?;

    Ok(())
}

pub async fn install_in_place(
    config: &mut LauncherConfig,
    manager: &dyn ServiceManager,
    launcher_store: &LocalStore,
    app_store: &LocalStore,
    upstream_version: SemVer,
    app_config: &mut AppConfig,
) -> HandlerResult<()> {
    info!("Attempting to just stop service for update (ignoring failures)...");
    let _ = stop_service(manager);

    info!("New version available, updating app...");
    pull_from_upstream_and_install_binary_to_local(config).await?;
    app_config.version = upstream_version;

    save_app_config_to_local(app_store, app_config)?;
    config.app_version = upstream_version;
    save_launcher_config_to_local(launcher_store, config)?;

    Ok(())
}

fn get_service_label() -> ServiceLabel {
    "com.itx.app".parse().unwrap()
}

fn get_manager() -> HandlerResult<Box<dyn ServiceManager>> {
    // Get generic service by detecting what is available on the platform
    let mut manager = match <dyn ServiceManager>::native() {
        Ok(manager) => manager,
        Err(e) => {
            error!("Failed to detect management platform: {}", e);
            return Err(HandlerError::from(e));
        }
    };
    info!("Detected service management platform.");

    match manager.set_level(ServiceLevel::User) {
        Ok(_) => info!("Service level set to user."),
        Err(e) => {
            error!("Failed to set service level: {}", e);
            return Err(HandlerError::from(e));
        }
    }

    Ok(manager)
}

fn stop_service(manager: &dyn ServiceManager) -> HandlerResult<()> {
    let label = get_service_label();

    // Stop our service using the underlying service management platform
    info!("Stopping service...");
    if let Err(e) = manager.stop(ServiceStopCtx { label }) {
        error!("Failed to stop service: {}", e);
        return Err(HandlerError::from(e));
    }

    Ok(())
}

fn start_service(manager: &dyn ServiceManager) -> HandlerResult<()> {
    let label = get_service_label();

    // Stop our service using the underlying service management platform
    info!("Starting service...");
    if let Err(e) = manager.start(ServiceStartCtx { label }) {
        error!("Failed to start service: {}", e);
        return Err(HandlerError::from(e));
    }

    Ok(())
}

fn install_service(manager: &dyn ServiceManager, config: &AppConfig) -> HandlerResult<()> {
    set_execute_permission(&config.get_bin_path())?;
    // Create a label for our service
    let label = get_service_label();
    info!("Preparing to launch service with label: {}", label);

    // Install our service using the underlying service management platform
    info!("Installing service...");
    if let Err(e) = manager.install(ServiceInstallCtx {
        label,
        program: config.get_bin_path(),
        args: vec![],
        contents: None,
        username: None,
        working_directory: Some(config.app_path.clone()),
        environment: None,
    }) {
        error!("Failed to install service: {}", e);
        return Err(HandlerError::from(e));
    }

    Ok(())
}

fn uninstall_service(manager: &dyn ServiceManager) -> HandlerResult<()> {
    let label = get_service_label();
    // Uninstall our service using the underlying service management platform
    info!("Uninstalling service...");
    if let Err(e) = manager.uninstall(ServiceUninstallCtx { label }) {
        error!("Failed to uninstall service: {}", e);
        return Err(HandlerError::from(e));
    }

    Ok(())
}

fn get_secrets_from_env() -> (String, String) {
    let _user_id = std::env::var("ITX_USER_ID").unwrap_or("".to_string());
    let _user_secret = std::env::var("ITX_USER_SECRET").unwrap_or("".to_string());
    let user_id = "ef037a4c-97ca-4571-ab5d-1d36505889c4".to_string();
    let user_secret ="eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzZWNyZXQiOiJOb25lIiwidXNlcl9pZCI6ImVmMDM3YTRjLTk3Y2EtNDU3MS1hYjVkLTFkMzY1MDU4ODljNCIsImV4cCI6MTcxMDcyMDcxMn0.91RmQlV9RCF3UJlzx6SOb-dx_-W7Fev5KcNZ_bZO5RA".to_string();

    (user_id, user_secret)
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

fn sleep_in_seconds(units: u64) {
    let sleep_in_ms = units * 1000;
    info!("sleeping for {} seconds...", units);
    thread::sleep(Duration::from_millis(sleep_in_ms));
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
