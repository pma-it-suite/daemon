use crate::models::{DeviceData, HandlerError};
use jfs;
use lazy_static::lazy_static;
use std::fs::File;
use std::io::prelude::*;

lazy_static! {
    //static ref DB: jfs::Store = jfs::Store::new_with_cfg(get_file_path(),jfs::Config{ single: true, pretty: true, ..Default::default()}).expect("should be able to create db store");
    static ref DB: jfs::Store = get_and_write().expect("should get store");
}

fn get_and_write() -> Result<jfs::Store, HandlerError> {
    let mut file = File::create(get_file_path())?;
    file.write_all(br#"{"q_id": {"user_id": "", "user_secret": "", "endpoint": "https://its.kdns.ooo:8080", "id": ""}}"#)?;
    Ok(jfs::Store::new_with_cfg(
        get_file_path(),
        jfs::Config {
            single: true,
            pretty: true,
            ..Default::default()
        },
    )?) // .expect("should be able to create db store")
}

pub fn deserialize_from_file() -> Result<DeviceData, HandlerError> {
    println!("getting...");
    match DB.get::<DeviceData>(&_query_id()) {
        Err(err) => {
            dbg!(&err);
            Err(HandlerError::DbError)
        }
        Ok(data) => Ok(data)
    }
}

pub fn update_or_add_to_file(data: DeviceData) -> Result<(), HandlerError> {
    println!("saving: {:#?}", &data);
    DB.save_with_id::<DeviceData>(&data, &_query_id())?;
    Ok(())
}

fn _query_id() -> String {
    "q_id".to_string()
}

fn get_file_path() -> String {
    "/tmp/inner_daemon-db.json".to_string()
}
