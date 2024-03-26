use super::define::{failure_response, success_response, InvokeResponse, Message};
use crate::toolbox::file;
use std::{
    fs::{self, File},
    io::{ Read, Write},
    path::Path, env
};
use tauri::InvokeError;
use serde_json::json;

#[tauri::command]
pub fn get_file_content(name: String) -> Result<String, InvokeError> {
    let file = File::open(name);
    let mut file = match file {
        Ok(f) => f,
        Err(err) => return Err(InvokeError::from(err.to_string())),
    };
    let mut content = String::new();
    if let Err(err) = file.read_to_string(&mut content) {
        return Err(InvokeError::from(err.to_string()));
    }
    Ok(content)
}

#[tauri::command]
pub fn write_file(name: String, content: String) -> InvokeResponse {
    match File::create(name) {
        Err(err) => {
            failure_response(Message::IoError(err))
        },
        Ok(mut buffer) => {
            match buffer.write(content.as_bytes()) {
                Ok(_) => success_response(json!(null)),
                Err(err) => failure_response(Message::IoError(err))
            }
        }
    }
}


#[tauri::command]
pub fn write_media_file(dir: String, name : String, content: Vec<u8>) -> InvokeResponse {
    let tmp_path = std::path::Path::new(&dir);
    if let Err(err) = std::fs::create_dir_all(&tmp_path) {
        return failure_response(Message::IoError(err));
    }
    let path_buf = tmp_path.join(&name);
    if let Ok(mut file) = File::create(path_buf.as_path()) {
        if let Ok(_) = file.write_all(&content) {
            return  success_response(json!(null));
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
    if let Err(err) =  fs::remove_file(String::from(file_path)) {
        failure_response(Message::IoError(err))
    } else {
        success_response(json!(null))
    }
}

#[tauri::command]
pub fn delete_dir(file_path: String) -> InvokeResponse {
    if let Err(err) =  fs::remove_dir_all(String::from(file_path)) {
        failure_response(Message::IoError(err))
    } else {
        success_response(json!(null))
    }
}

#[tauri::command]
pub fn rename_file(file_path: String, new_file_path : String) -> InvokeResponse {
    if let Err(err) =  fs::rename(String::from(file_path), String::from(new_file_path)) {
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


