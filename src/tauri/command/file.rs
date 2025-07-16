use super::define::{failure_response, success_response, InvokeResponse, Message};
use crate::toolbox::file;
use serde_json::json;
use serde_json::Value;
use std::{
    env,
    fs::{self, File},
    io::{Read, Write},
    path::Path,
};

#[tauri::command]
pub fn get_file_content(name: String) -> InvokeResponse {
    let file = File::open(name);
    match file {
        Ok(mut f) => {
            let mut content = String::new();
            if let Err(err) = f.read_to_string(&mut content) {
                return failure_response(Message::IoError(err));
            }
            success_response(Value::String(content))
        }
        Err(err) => return failure_response(Message::IoError(err)),
    }
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
pub fn write_media_file(dir: String, name: String, content: Vec<u8>) -> InvokeResponse {
    let tmp_path = std::path::Path::new(&dir);
    if let Err(err) = std::fs::create_dir_all(&tmp_path) {
        return failure_response(Message::IoError(err));
    }
    let path_buf = tmp_path.join(&name);
    if let Ok(mut file) = File::create(path_buf.as_path()) {
        if let Ok(_) = file.write_all(&content) {
            return success_response(json!(null));
        }
    }
    success_response(json!(null))
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
