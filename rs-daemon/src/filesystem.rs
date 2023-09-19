use crate::models::{FsResult, ProcessData};
use jfs;
use serde_json;
use std::fs::File;
use std::io::prelude::*;
use lazy_static::lazy_static;

lazy_static! {
    /// This is an example for using doc comment attributes
    static ref DB: jfs::Store = jfs::Store::new_with_cfg("/tmp/var/rs-daemon/db.json",jfs::Config{ single: true, pretty: true, ..Default::default()}).expect("should be able to create db store");
}

pub fn save_process(data: &ProcessData) -> FsResult<()> {
    write_to_file(IOContent::Object(data))?;
    Ok(())
}

pub struct Query<'a> {
    _name: Option<&'a str>,
    id: Option<&'a str>,
}

pub fn get_process(query: Query) -> FsResult<ProcessData> {
    if query.id.is_none() {
        panic!("must have id for query");
        }
    let process = deserialize_from_file(query.id.unwrap())?;
    Ok(process)
}

fn deserialize_from_file(id: &str) -> FsResult<ProcessData> {
    let data = DB.get::<ProcessData>(id)?;
    Ok(data)
}

fn get_file_path() -> String {
    "/tmp/var/rs-daemon/db.json".to_string()
}

pub fn _read_from_file() -> FsResult<String> {
    let file_path_str = get_file_path();
    let mut contents = String::new();

    let mut file = File::open(file_path_str)?;
    file.read_to_string(&mut contents)?;

    Ok(contents)
}

pub enum IOContent<'a> {
    Raw(&'a str),
    Object(&'a ProcessData)
}

pub fn write_to_file(content: IOContent) -> FsResult<()> {
    match content {
        IOContent::Raw(data) => {
            let serialized_data: serde_json::Value = serde_json::from_str(data)?;
            DB.save(&serialized_data)?
        }
        IOContent::Object(data) => DB.save_with_id(data, &data.pid.to_string())?
    };
    Ok(())
}
