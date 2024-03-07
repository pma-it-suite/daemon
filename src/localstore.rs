use crate::models::HandlerError;
use jfs::{self};
use lazy_static::lazy_static;
use log::{debug, info};
use std::collections::HashMap;
use std::io::Write as _;

use std::sync::Mutex;
use tempdir::TempDir;

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

lazy_static! {
    static ref HANDLE: Mutex<Result<jfs::Store, HandlerError>> = Mutex::new(get_handle());
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

fn get_handle() -> Result<jfs::Store, HandlerError> {
    let file_path = get_default_filepath();
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

    use crate::{localstore::get_default_filepath, test_commons::before_each};

    #[test]
    fn test_get_handle_creates_file() {
        before_each();
        let test_path = get_default_filepath();
        let mut does_exist = std::path::Path::new(&test_path).exists();
        assert!(!does_exist);

        let result = super::get_handle();
        dbg!(&result);
        assert!(result.is_ok());

        does_exist = std::path::Path::new(&test_path).exists();
        assert!(does_exist);
    }
}
