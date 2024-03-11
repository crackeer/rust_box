use serde::{Deserialize, Serialize};
use serde_json::json;
use tauri::InvokeError;
use std::io::Error;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct InvokeResponse {
    pub success: bool,
    pub message: String,
    pub data : serde_json::Value,
}

pub enum Message {
    String(String),
    InvokeError(InvokeError),
    IoError(Error),
}

pub fn success_response(value : serde_json::Value) -> InvokeResponse {
    InvokeResponse {
        success: true,
        message: "success".to_string(),
        data: value,
    }
}

pub fn failure_response(msg : Message) -> InvokeResponse {
    match msg {
        Message::String(msg) =>  InvokeResponse {
            success: false,
            message: msg,
            data: json!({}),
        },
        Message::InvokeError(err) => {
            //let value : Value = err.fmt("{}");
            //let value : Value = err as Value;
            InvokeResponse {
                success: false,
                message: format!("{:?}", err),
                data: json!({}),
            }
        },
        Message::IoError(err) => {
            InvokeResponse {
                success: false,
                message: err.to_string(),
                data: json!({}),
            }
        }
    }
   
}

