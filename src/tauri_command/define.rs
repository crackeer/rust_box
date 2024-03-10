use serde::{Deserialize, Serialize};
use serde_json;
use serde_json::json;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct InvokeResponse {
    pub success: bool,
    pub message: String,
    pub data : serde_json::Value,
}

pub fn success_response(value : serde_json::Value) -> InvokeResponse {
    InvokeResponse {
        success: true,
        message: "success".to_string(),
        data: value,
    }
}

pub fn failure_response(msg : String) -> InvokeResponse {
    InvokeResponse {
        success: false,
        message: msg,
        data: json!({}),
    }
}

