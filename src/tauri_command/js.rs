use super::define::{failure_response, success_response, InvokeResponse, Message};
use serde_json::json;
use std::process::Command;

use tauri;

#[tauri::command]
pub fn run_js_code(code: String) -> InvokeResponse {
    match Command::new("node").arg("-e").arg(code).output() {
        Err(err) => failure_response(Message::String(err.to_string())),
        Ok(output) => {
            if !output.status.success() {
                let stderr: Vec<u8> = output.stderr.into();
                return failure_response(Message::String(
                    String::from_utf8_lossy(&stderr).to_string(),
                ));
            }
            let stdout: Vec<u8> = output.stdout.into();
            success_response(json!({
                "output" : String::from_utf8_lossy(&stdout).to_owned(),
            }))
        }
    }
}
