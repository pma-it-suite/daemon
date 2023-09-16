use crate::models::{FsResult, ProcessData};
use serde_json;
use std::fs::File;
use std::io::prelude::*;
use std::io::{self, BufReader, BufWriter};
use std::path::Path;

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

pub fn get_process_filepath() -> String {
    "/Users/felipearce/Desktop/projects/shellhacks2023/daemon/rs-daemon/inner_daemon/target/debug/inner_daemon".to_string()
}

pub fn get_input_filepath() -> String {
    "/Users/felipearce/Desktop/projects/shellhacks2023/daemon/rs-daemon/inner_daemon/in.txt"
        .to_string()
}

pub fn get_output_filepath() -> String {
    "/Users/felipearce/Desktop/projects/shellhacks2023/daemon/rs-daemon/inner_daemon/out.txt"
        .to_string()
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

pub fn get_buf_reader_handle(filepath: &str) -> io::Result<BufReader<File>> {
    let file = get_or_create(filepath)?;
    Ok(BufReader::new(file))
}

pub fn get_buf_writer_handle(filepath: &str) -> io::Result<BufWriter<File>> {
    let file = get_or_create(filepath)?;
    Ok(BufWriter::new(file))
}

fn get_or_create(file_path_str: &str) -> io::Result<File> {
    let file_path = Path::new(file_path_str);

    let file = match file_path.exists() {
        true => File::open(file_path_str)?,
        false => File::create(file_path_str)?,
    };
    Ok(file)
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
