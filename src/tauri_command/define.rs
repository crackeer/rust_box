use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct InvokeResponse {
    pub success: bool,
    pub message: String,
    pub data : serde_json::Value,
}

