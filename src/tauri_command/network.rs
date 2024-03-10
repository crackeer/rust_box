use super::define::{InvokeResponse, success_response};
use crate::toolbox::network;
use serde_json::json;

#[tauri::command]
pub fn get_local_addr() -> InvokeResponse {
    if let Some(addr) = network::get_local_addr() {
        return success_response(json!({
            "addr": addr
        }));
    }
    success_response(json!({
        "addr": ""
    }))
}