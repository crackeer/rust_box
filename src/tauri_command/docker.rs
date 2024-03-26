use super::define::{failure_response, success_response, InvokeResponse, Message};
use bollard::errors::Error;
use bollard::image::ListImagesOptions;
use bollard::Docker;
use bollard::API_DEFAULT_VERSION;
use serde_json::json;
use std::collections::HashMap;
use std::env;
use std::io::Write;
use std::path::PathBuf;
use std::fs;
use std::fs::File;
use std::path::Path;
use futures::stream::StreamExt;

#[tauri::command]
pub async fn docker_image_list() -> InvokeResponse {
    match get_docker() {
        Err(err) => failure_response(Message::String(err.to_string())),
        Ok(docker) => {
            match docker
                .list_images(Some(ListImagesOptions::<String> {
                    all: true,
                    filters: HashMap::new(),
                    digests: true,
                }))
                .await
            {
                Ok(list) => {
                    return success_response(json!(list));
                }
                Err(err) => failure_response(Message::String(err.to_string())),
            }
        }
    }
}

fn get_docker() -> Result<Docker, Error> {
    if let Some(sock_path) = get_user_docker_sock_path() {
        return Docker::connect_with_socket(sock_path.to_str().unwrap(), 120, API_DEFAULT_VERSION);
    }
    Docker::connect_with_local_defaults()
}

fn get_user_docker_sock_path() -> Option<PathBuf> {
    match env::home_dir() {
        Some(path) => {
            let tmp_path = path.join(&".docker/run/docker.sock");
            if fs::metadata(tmp_path.as_path()).is_ok() {
                return Some(tmp_path);
            }
            None
        }
        None => None,
    }
}


#[tauri::command]
pub async fn save_images_to_local(image_names : Vec<String>, local_dir: String) -> InvokeResponse {
    if let Err(err) = std::fs::create_dir_all(&local_dir) {
        return failure_response(Message::String(err.to_string()));
    }

    match get_docker() {
        Err(err) => {
            return failure_response(Message::String(err.to_string()));
        },
        Ok(docker) => {
            let mut local_files : Vec<String> = vec![];
            for name in image_names.iter() {
                let tmp_path = Path::new(&local_dir).join(get_image_file_name(&name));
                let mut tmp_file = File::create(&tmp_path).unwrap();
                let mut stream = docker.export_image(&name);
                while let Some(item) = stream.next().await {
                    if let Ok(data) = item {
                        if let Err(err) = tmp_file.write(&data) {
                            return failure_response(Message::String(err.to_string()));
                        }
                    }
                }
                local_files.push(tmp_path.to_str().unwrap().to_string());
            }
            success_response(json!(local_files))
        }
    }

}

fn get_image_file_name(name : &str) -> String {
    let parts = name.split(&":").collect::<Vec<&str>>();
    if parts.len() >= 1 {
        return format!("{}.tar", parts[0]);
    }
    return name.to_string();
}