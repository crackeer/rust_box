use super::define::{InvokeResponse, success_response, failure_response, Message};
use bollard::Docker;
use bollard::API_DEFAULT_VERSION;
use bollard::image::ListImagesOptions;
use std::default::Default;
use serde_json::json;

#[tauri::command]
pub async fn docker_image_list() -> InvokeResponse {
    match Docker::connect_with_socket(&"/un/docker.sock", 120, API_DEFAULT_VERSION) {
        Err(err) => failure_response(Message::String(err.to_string())),
        Ok(ref docker) => {
            match  docker.list_images(Some(ListImagesOptions::<String> {
                all: true,
                ..Default::default()
            })).await {
                Ok(list) => {
                    return success_response(json!(list));
                },
                Err(err) =>  failure_response(Message::String(err.to_string()))
            }
        }
    }
}
