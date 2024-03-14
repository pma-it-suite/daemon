#![feature(async_closure)]
#![feature(never_type)]

pub fn main() {
}

pub mod models {
    use thiserror::Error;

    pub type Id = String;

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
}

pub mod localstore {
    use crate::models::HandlerError;
    use jfs::{self};
    use lazy_static::lazy_static;
    use log::{debug, info};
    use std::collections::HashMap;

    use std::io::Write as _;

    use std::sync::Mutex;
    use tempdir::TempDir;

    pub fn get_default_filepath() -> String {
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
            return Some(
                TempDir::new("test-localstore").expect("should be able to create tempdir"),
            );
        }
        None
    }

    pub fn get_handle() -> Result<jfs::Store, HandlerError> {
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

    fn get_user_id() -> String {
        "ee9470de-54a4-419c-b34a-ba2fa18731d8".to_string()
    }

    fn init_db(db: &jfs::Store) -> Result<(), HandlerError> {
        debug!("filepath for store on init is: {:#?}", &db.path());
        let key = "user_id";
        let resp = query_internal(db, key);
        if resp.is_err() || resp?.is_none() {
            debug!("user_id not set, setting to default");
            let user_id = get_user_id();
            db.save_with_id(&user_id, key)?;
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

        use lazy_static::lazy_static;

        use crate::{
            localstore::{get_default_filepath, get_handle, get_user_id},
            test_commons::before_each_fs,
        };
        use std::{collections::HashMap, io::Write as _, sync::Mutex};

        fn does_default_file_exist() -> bool {
            let test_path = get_default_filepath();
            std::path::Path::new(&test_path).exists()
        }

        fn get_file_data() -> String {
            let test_path = get_default_filepath();
            std::fs::read_to_string(test_path).unwrap()
        }

        fn does_file_contain(data: &str) -> bool {
            let file_data = get_file_data();
            file_data.contains(data)
        }

        fn write_to_file(data: &str) {
            let file_path = get_default_filepath();
            std::fs::File::create(&file_path).unwrap();
            std::fs::OpenOptions::new()
                .write(true)
                .open(&file_path)
                .unwrap()
                .write_all(data.as_bytes())
                .unwrap();
        }

        fn get_test_key_val() -> (String, String) {
            ("key1".to_string(), "value1".to_string())
        }

        fn get_test_data() -> HashMap<String, String> {
            let mut map = HashMap::new();
            vec![get_test_key_val()].iter().for_each(|(k, v)| {
                map.insert(k.to_string(), v.to_string());
            });
            map
        }

        lazy_static! {
            static ref LOCK: Mutex<()> = Mutex::new(());
        }

        #[test]
        fn test_get_handle_creates_file() {
            let _tmp = LOCK.lock().unwrap();
            before_each_fs();
            assert!(!does_default_file_exist());

            let result = super::get_handle();
            assert!(result.is_ok());

            assert!(does_default_file_exist());
        }

        #[test]
        fn test_db_init_happens_if_file_empty() {
            let _tmp = LOCK.lock().unwrap();
            before_each_fs();

            assert!(!does_default_file_exist());

            let result = super::get_handle();
            assert!(result.is_ok());

            assert!(does_default_file_exist());
            dbg!(&get_file_data());
            assert!(does_file_contain(&get_user_id()));
        }

        #[test]
        fn test_db_init_does_not_happens_if_file_populated() {
            let _tmp = LOCK.lock().unwrap();
            before_each_fs();

            let test_id = "testid";
            let test_data = r#"{"user_id": ""#.to_owned() + test_id + r#""}"#;
            write_to_file(&test_data);

            assert!(does_default_file_exist());
            assert!(does_file_contain(&test_data));

            let result = super::get_handle();
            assert!(result.is_ok());

            assert!(does_default_file_exist());
            assert!(!does_file_contain(&get_user_id()));
            assert!(does_file_contain(&test_data));
        }

        #[test]
        fn test_insert_works() {
            let _tmp = LOCK.lock().unwrap();
            before_each_fs();

            let data = get_test_data();
            assert!(!does_default_file_exist());

            let _ = get_handle().unwrap();
            let result = super::write_data(data);
            assert!(result.is_ok());

            assert!(does_default_file_exist());
            let (test_key, test_val) = get_test_key_val();
            assert!(does_file_contain(test_key.as_str()));
            assert!(does_file_contain(test_val.as_str()));
        }

        #[test]
        fn test_insert_replaces_existing_key() {
            let _tmp = LOCK.lock().unwrap();
            before_each_fs();

            let (test_key, test_val) = get_test_key_val();
            let second_test_val = "testval2";
            let mut data = get_test_data();
            data.insert(test_key.clone(), second_test_val.to_string());
            assert!(!does_default_file_exist());

            let _ = get_handle().unwrap();
            let result = super::write_data(data);
            assert!(result.is_ok());

            assert!(does_default_file_exist());
            assert!(does_file_contain(test_key.as_str()));
            assert!(!does_file_contain(test_val.as_str()));
            assert!(does_file_contain(second_test_val));
        }

        #[test]
        fn test_query_works() {
            let _tmp = LOCK.lock().unwrap();
            before_each_fs();

            let data = get_test_data();
            assert!(!does_default_file_exist());

            let _ = get_handle().unwrap();
            let result = super::write_data(data);
            assert!(result.is_ok());

            assert!(does_default_file_exist());
            let (test_key, test_val) = get_test_key_val();
            assert!(does_file_contain(test_key.as_str()));
            assert!(does_file_contain(test_val.as_str()));

            let query_result = super::query_data(test_key.as_str());
            assert!(query_result.is_ok());
            let response = query_result.unwrap();
            assert!(response.is_some());
            assert_eq!(response.unwrap(), test_val);
        }

        #[test]
        fn test_query_fails_when_missing_key() {
            let _tmp = LOCK.lock().unwrap();
            before_each_fs();

            assert!(!does_default_file_exist());

            let _ = get_handle().unwrap();

            assert!(does_default_file_exist());
            let (test_key, _) = get_test_key_val();
            assert!(!does_file_contain(test_key.as_str()));

            let query_result = super::query_data(test_key.as_str());
            assert!(query_result.is_err());
        }
    }
}

