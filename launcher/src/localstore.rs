use crate::models::{HandlerError, HandlerResult};
use jfs::{self};
use log::{debug, info};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::collections::HashMap;

use std::io::Write as _;

use std::path::PathBuf;

pub struct LocalStore {
    db: jfs::Store,
    path: PathBuf,
}

impl LocalStore {
    pub fn new(path: PathBuf) -> HandlerResult<Self> {
        Ok(Self {
            db: Self::get_handle(path.clone())?,
            path,
        })
    }

    pub fn from<T: Serialize>(path: PathBuf, config: &T) -> HandlerResult<Self> {
        let store = Self::new(path)?;
        store.overwrite_current_data(config)?;
        Ok(store)
    }

    pub fn get_all_data<T: DeserializeOwned>(&self) -> HandlerResult<T> {
        // read all data through fs api and return it
        let file_path = &self.path;
        let file_data = std::fs::read_to_string(file_path)?;
        let data: T = serde_json::from_value(serde_json::Value::String(file_data))?;
        Ok(data)
    }

    pub fn overwrite_current_data<T: Serialize>(&self, config: &T) -> HandlerResult<()> {
        let file_path = &self.path;
        let data = serde_json::to_string(config)?;
        std::fs::File::create(file_path)?;
        std::fs::OpenOptions::new()
            .write(true)
            .open(file_path)?
            .write_all(data.as_bytes())?;
        Ok(())
    }

    pub fn write_single(&self, data: &String, key: &str) -> HandlerResult<()> {
        info!("writing data for key: {}", key);
        self.db.save_with_id(data, key)?;
        Ok(())
    }

    pub fn write_data(&self, data: HashMap<String, String>) -> HandlerResult<()> {
        data.keys().for_each(|key| {
            self.write_single(&data[key], key).unwrap();
        });
        Ok(())
    }

    pub fn query(&self, key: &str) -> HandlerResult<Option<String>> {
        info!("querying data for key: {}", key);
        let data = self.db.get(key)?;
        Ok(data)
    }

    fn get_handle(file_path: PathBuf) -> Result<jfs::Store, HandlerError> {
        // if file dir doesnt exist make it
        if !file_path.parent().unwrap().exists() {
            std::fs::create_dir_all(file_path.parent().unwrap())?;
        }
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
        Ok(db)
    }
}
