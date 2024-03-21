use super::define::{failure_response, success_response, InvokeResponse, Message};
use bollard::image::ListImagesOptions;
use bollard::{Docker};
use bollard::errors::Error;
use bollard::API_DEFAULT_VERSION;
use serde_json::json;
use std::default::Default;
use std::env;
use std::path::Path;

#[tauri::command]
pub async fn docker_image_list() -> InvokeResponse {
    match Docker::connect_with_socket(&"/un/docker.sock", 120, API_DEFAULT_VERSION) {
        Err(err) => failure_response(Message::String(err.to_string())),
        Ok(ref docker) => {
            match docker
                .list_images(Some(ListImagesOptions::<String> {
                    all: true,
                    ..Default::default()
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

fn get_docker() -> Result<Docker, String> {
    if let Ok(docker) = Docker::connect_with_local_defaults() {
        return Ok(docker);
    }

    match get_user_docker_sock() {
        Ok(dir) => {
            let user_path = Path::new(&dir).join(&".docker/docker.sock");
            if let Ok(docker) = Docker::connect_with_socket(user_path.to_str().unwrap(), 120, API_DEFAULT_VERSION) {
                return Ok(docker);
            }
            return Err(String::from("Sim"));
        },
        Err(err) => Err(err),
    }
}

fn get_user_docker_sock() -> Result<String, String> {
    match env::home_dir() {
        Some(path) => Ok(path.to_str().unwrap().to_string()),
        None => Err(String::from("no home directory")),
    }
}
