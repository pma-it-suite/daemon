use crate::models::{FsResult, ProcessData};
use jfs;
use serde_json;
use std::fs::File;
use std::io::prelude::*;
use std::io::{self};
use std::path::Path;
use lazy_static::lazy_static;

lazy_static! {
    /// This is an example for using doc comment attributes
    static ref DB: jfs::Store = jfs::Store::new_with_cfg("/Users/felipearce/Desktop/projects/shellhacks2023/daemon/rs-daemon/db.json",jfs::Config{ single: true, ..Default::default()}).expect("should be able to create db store");
}

pub fn make_process_file(data: ProcessData) -> FsResult<()> {
    write_to_file(IOContent::Object(data))?;
    Ok(())
}

pub fn get_process() -> FsResult<ProcessData> {
    let process = deserialize_from_file()?;
    Ok(process)
}

fn deserialize_from_file() -> FsResult<ProcessData> {
    let contents = read_from_file()?;
    let data = serde_json::from_str(&contents)?;
    Ok(data)
}

fn get_file_path() -> String {
    "/Users/felipearce/Desktop/projects/shellhacks2023/daemon/rs-daemon/db.json".to_string()
}

pub fn get_process_filepath() -> String {
    "/Users/felipearce/Desktop/projects/shellhacks2023/daemon/rs-daemon/inner_daemon/target/debug/inner_daemon".to_string()
}

pub fn read_from_file() -> FsResult<String> {
    let file_path_str = get_file_path();
    let mut contents = String::new();

    let mut file = File::open(file_path_str)?;
    file.read_to_string(&mut contents)?;

    Ok(contents)
}

fn get_or_create(file_path_str: &str) -> io::Result<File> {
    let file_path = Path::new(file_path_str);

    let file = match file_path.exists() {
        true => File::open(file_path_str)?,
        false => File::create(file_path_str)?,
    };
    Ok(file)
}

pub enum IOContent<'a> {
    Raw(&'a str),
    Object(ProcessData)
}

pub fn write_to_file(content: IOContent) -> FsResult<()> {
    match content {
        IOContent::Raw(data) => {
            let serialized_data: serde_json::Value = serde_json::from_str(data)?;
            DB.save(&serialized_data)?
        }
        IOContent::Object(data) => DB.save(&data)?
    };
    Ok(())
}
