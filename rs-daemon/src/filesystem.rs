use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use crate::models::{ProcessData, FsResult};
use serde_json;

pub fn make_process_file(data: ProcessData) -> FsResult<()> {
    write_to_file(&data.to_output())?;
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
    "/Users/felipearce/Desktop/projects/shellhacks2023/daemon/rs-daemon/test.txt".to_string()
}

pub fn read_from_file() -> FsResult<String> {
    let file_path_str = get_file_path();
    let mut contents = String::new();

    let mut file = File::open(file_path_str)?;
    file.read_to_string(&mut contents)?;

    Ok(contents)
}

pub fn write_to_file(content: &str) -> FsResult<()> {
    let file_path_str = get_file_path();
    let file_path = Path::new(&file_path_str);

    let mut file = match file_path.exists() {
        true => File::open(file_path_str)?,
        false => File::create(file_path_str)?,
    };
    file.write(content.as_bytes())?;
    Ok(())
}
