use jfs;
use lazy_static::lazy_static;
use crate::models::{DeviceData, HandlerError};

lazy_static! {
    /// This is an example for using doc comment attributes
    static ref DB: jfs::Store = jfs::Store::new_with_cfg(get_file_path(),jfs::Config{ single: true, pretty: true, ..Default::default()}).expect("should be able to create db store");
}

pub fn deserialize_from_file() -> Result<DeviceData, HandlerError> {
    let data = DB.get::<DeviceData>(&_query_id())?;
    Ok(data)
}

pub fn update_or_add_to_file(data: DeviceData) -> Result<(), HandlerError> {
    DB.save::<DeviceData>(&data)?;
    Ok(())
}

fn _query_id() -> String {
    "q_id".to_string()
}

fn get_file_path() -> String {
    "/tmp/var/inner_daemon/db.json".to_string()
}

