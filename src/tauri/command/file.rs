use super::define::{failure_response, success_response, InvokeResponse, Message};
use crate::toolbox::file;
use serde_json::json;
use std::{
    env,
    fs::{self, File},
    io::{Read, Write},
    path::Path,
};

#[tauri::command]
pub fn get_file_content(name: String) -> Result<String, String> {
    let mut file = File::open(name).map_err(|e| e.to_string())?;
    let mut content = String::new();
    file.read_to_string(&mut content).map_err(|e| e.to_string())?;
    Ok(content)
}

#[tauri::command]
pub fn write_file(name: String, content: String) -> InvokeResponse {
    match File::create(name) {
        Err(err) => failure_response(Message::IoError(err)),
        Ok(mut buffer) => match buffer.write(content.as_bytes()) {
            Ok(_) => success_response(json!(null)),
            Err(err) => failure_response(Message::IoError(err)),
        },
    }
}

#[tauri::command]
pub fn write_blob_file(file_path: String, content: Vec<u8>) -> Result<(), String> {
    file::create_file_parent_directory(&file_path).map_err(|e| e.to_string())?;
    let mut tmp_file = File::create(file_path).map_err(|e| e.to_string())?;
    tmp_file.write_all(&content).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn create_dir(file_path: String) -> InvokeResponse {
    if let Err(err) = fs::create_dir_all(String::from(file_path)) {
        failure_response(Message::IoError(err))
    } else {
        success_response(json!(null))
    }
}

#[tauri::command]
pub fn create_file(file_path: String) -> InvokeResponse {
    if let Err(err) = File::create(String::from(file_path)) {
        failure_response(Message::IoError(err))
    } else {
        success_response(json!(null))
    }
}

#[tauri::command]
pub fn delete_file(file_path: String) -> InvokeResponse {
    if let Err(err) = fs::remove_file(String::from(file_path)) {
        failure_response(Message::IoError(err))
    } else {
        success_response(json!(null))
    }
}

#[tauri::command]
pub fn delete_dir(file_path: String) -> InvokeResponse {
    if let Err(err) = fs::remove_dir_all(String::from(file_path)) {
        failure_response(Message::IoError(err))
    } else {
        success_response(json!(null))
    }
}

#[tauri::command]
pub fn rename_file(file_path: String, new_file_path: String) -> InvokeResponse {
    if let Err(err) = fs::rename(String::from(file_path), String::from(new_file_path)) {
        failure_response(Message::IoError(err))
    } else {
        success_response(json!(null))
    }
}

#[tauri::command]
pub fn file_exists(file_path: String) -> InvokeResponse {
    success_response(json!({
        "exists" : Path::new(&file_path).exists(),
    }))
}

#[tauri::command]
pub fn list_folder(file_path: String) -> InvokeResponse {
    success_response(json!(file::simple_read_dir(file_path)))
}

#[tauri::command]
pub fn get_temp_dir() -> InvokeResponse {
    let dir = env::temp_dir();
    success_response(json!({
        "temp_dir" : dir,
    }))
}

#[tauri::command]
pub fn create_jsonp_file(src_file: String, dest_file: String, hash_code: String) -> InvokeResponse {
    match file::create_jsonp_file(src_file.as_str(), dest_file.as_str(), hash_code.as_str()) {
        Ok(_) => success_response(json!(null)),
        Err(err) => failure_response(Message::String(err)),
    }
}
