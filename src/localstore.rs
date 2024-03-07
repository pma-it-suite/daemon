use crate::models::HandlerError;
use jfs::{self};
use lazy_static::lazy_static;
use log::{debug, info};
use std::collections::HashMap;
use std::io::Write as _;
use std::path::PathBuf;
use std::sync::Mutex;
use tempdir::TempDir;

pub struct StoreConfig {
    pub path: PathBuf,
}

impl StoreConfig {
    pub fn new(path: PathBuf) -> Self {
        StoreConfig { path }
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    pub fn append_path(&mut self, path: &str) {
        self.path.push(path);
    }
}

fn get_default_filepath() -> String {
    if cfg!(test) {
        let filepath = "localstore.json".to_string();
        return TEMPDIR
            .lock()
            .as_ref()
            .unwrap()
            .as_ref()
            .unwrap()
            .path()
            .join(filepath)
            .display()
            .to_string();
    }
    "localstore.json".to_string()
}

impl Default for StoreConfig {
    fn default() -> Self {
        StoreConfig {
            path: PathBuf::from(get_default_filepath()),
        }
    }
}

lazy_static! {
    static ref HANDLE: Mutex<Result<jfs::Store, HandlerError>> = Mutex::new(get_handle_inner());
}

lazy_static! {
    static ref TEMPDIR: Mutex<Option<TempDir>> = Mutex::new(get_tempdir());
}

fn get_tempdir() -> Option<TempDir> {
    if cfg!(test) {
        return Some(TempDir::new("test-localstore").expect("should be able to create tempdir"));
    }
    None
}

fn get_handle_inner() -> Result<jfs::Store, HandlerError> {
    let config = StoreConfig::default();
    get_handle(config)
}

fn get_handle(config: StoreConfig) -> Result<jfs::Store, HandlerError> {
    let file_path = config.path;
    // if file doesnt exist make it
    if !std::path::Path::new(&file_path).exists() {
        std::fs::File::create(&file_path)?;
        // and write a {} to it
        std::fs::OpenOptions::new()
            .write(true)
            .open(&file_path)?
            .write_all(b"{}")?;
    }

    info!("creating or getting store from path: {:#?}", &file_path);
    let result = jfs::Store::new_with_cfg(
        file_path,
        jfs::Config {
            single: true,
            pretty: true,
            ..Default::default()
        },
    );
    if result.is_err() {
        debug!("error in creating db {:#?}", &result);
        return Err(HandlerError::IoError(result.err().unwrap()));
    }
    let db = result.unwrap();
    debug!("store created: {:#?}", &db);
    init_db(&db)?;
    Ok(db)
}

fn init_db(db: &jfs::Store) -> Result<(), HandlerError> {
    debug!("filepath for store on init is: {:#?}", &db.path());
    let key = "user_id";
    let resp = query_internal(db, key);
    if resp.is_err() || resp?.is_none() {
        debug!("user_id not set, setting to default");
        let user_id = "ee9470de-54a4-419c-b34a-ba2fa18731d8";
        db.save_with_id(&user_id.to_string(), key)?;
        info!("user_id set to default: {}", user_id);
    }
    Ok(())
}

pub fn write_single(data: &String, key: &str) -> Result<(), HandlerError> {
    info!("writing data for key: {}", key);
    let binding = HANDLE.lock().unwrap();
    let handle = binding.as_ref().unwrap();
    handle.save_with_id(data, key)?;
    Ok(())
}

pub fn write_data(data: HashMap<String, String>) -> Result<(), HandlerError> {
    data.keys().for_each(|key| {
        write_single(&data[key], key).unwrap();
    });
    Ok(())
}

fn query_internal(db: &jfs::Store, key: &str) -> Result<Option<String>, HandlerError> {
    let data = db.get(key)?;
    Ok(data)
}

pub fn query_data(key: &str) -> Result<Option<String>, HandlerError> {
    info!("querying data for key: {}", key);
    let binding = HANDLE.lock().unwrap();
    let handle = binding.as_ref().unwrap();
    query_internal(handle, key)
}

#[cfg(test)]
mod test {
    use tempdir::TempDir;

    use crate::{
        localstore::{get_default_filepath, StoreConfig},
        test_commons::before_each,
    };

    fn get_tempdir() -> TempDir {
        TempDir::new("test-localstore").expect("should be able to create tempdir")
    }

    #[test]
    fn test_get_handle_creates_file() {
        before_each();
        let dir = get_tempdir();
        let mut path = StoreConfig::new(dir.path().to_path_buf());
        path.append_path(&get_default_filepath());

        let test_path = path.path().clone();
        let mut does_exist = std::path::Path::new(&test_path).exists();
        assert!(!does_exist);

        let result = super::get_handle(path);
        dbg!(&result);
        assert!(result.is_ok());

        does_exist = std::path::Path::new(&test_path).exists();
        assert!(does_exist);
    }
}
