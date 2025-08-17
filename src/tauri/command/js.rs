use super::define::{failure_response, success_response, InvokeResponse, Message};
use serde_json::json;
use std::{ process::Command};
use tauri;
use quickjs_runtime::builder::QuickJsRuntimeBuilder;
use quickjs_runtime::jsutils::Script;
use quickjs_runtime::values::JsValueFacade;

#[tauri::command]
pub fn run_js_code(node_path: String, code: String) -> InvokeResponse {
    match Command::new(&node_path).arg("-e").arg(code).output() {
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

/*
#[tauri::command]
pub async fn run_js_script(script_code: String) -> Result<String, String> {
    let runtime = QuickJsRuntimeBuilder::new().build();
    let realm_name = "main";
    let script = Script::new(realm_name, &script_code);
    let result = runtime.eval(None, script).await.map_err(|e| e.to_string())?;
    Ok(result)
}
     */


#[tauri::command]
pub async fn run_quick_js_code(script_code: String) -> Result<String, String> {
    let context = QuickJsRuntimeBuilder::new().build();
    let result = context.eval_sync(None, Script::new("file://main.js", &script_code)).map_err(|e| e.to_string())?;
    Ok( result.stringify())
}