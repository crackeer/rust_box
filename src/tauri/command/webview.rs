use std::ffi::OsStr;

use headless_chrome::{Browser, LaunchOptionsBuilder};
use serde_json::Value;
use tauri::{webview::WebviewWindowBuilder, AppHandle, Listener, Url, WebviewUrl};
use tokio::time::Duration;
#[tauri::command]
pub async fn open_new_webview(app_handle: AppHandle, url: &str) -> Result<(), String> {
    let ok_url = Url::parse(&url).map_err(|e| format!("解析URL失败: {}", e))?;
    // 1. 创建一个新的、可能不可见的 WebView 窗口
    println!("url: {}", url);
    let webview_window = WebviewWindowBuilder::new(
        &app_handle,
        "headless_feed_fetcher",
        WebviewUrl::External(ok_url),
    )
    .visible(true) // 确保窗口不可见
    .build()
    .map_err(|e| format!("创建WebView窗口失败: {}", e))?;
    /*
       webview_window.once("loaded", |_event| {
           println!("loaded");
       });
       tokio::time::sleep(Duration::from_secs(5)).await;
       let result_json = webview_window
           .eval(script)
           .map_err(|e| format!("执行脚本失败: {}", e))?;
    */
    Ok(())
}

#[tauri::command]
pub async fn eval_js_on_page(url: &str, scripts: Vec<String>) -> Result<Vec<Value>, String> {
    println!("url: {}", url);
    let browser = Browser::new(
        LaunchOptionsBuilder::default()
            .headless(true)
            .build()
            .unwrap(),
    )
    .map_err(|e| format!("创建浏览器失败: {}", e))?;

    let tab = browser
        .new_tab()
        .map_err(|e| format!("创建标签页失败: {}", e))?;
    tab.navigate_to(url)
        .map_err(|e| format!("导航到URL失败: {}", e))?;

    tab.wait_until_navigated()
        .map_err(|e| format!("等待导航完成失败: {}", e))?;
    println!("navigated");
    tokio::time::sleep(Duration::from_secs(3)).await;
    let mut list: Vec<Value> = Vec::new();
    for script in scripts {
        println!("evaluate: {}", script);
        let value: Value = match tab.evaluate(script.as_str(), false) {
            Ok(v) => v.value.map(|v| v).unwrap_or_default(),
            Err(e) => Value::String(format!("{}", e)),
        };
        list.push(value);
    }
    println!("list: {:?}", list);
    //tab.close(true).map_err(|e| format!("关闭标签页失败: {}", e))?;
    //tokio::time::sleep(Duration::from_secs(1150)).await;
    Ok(list)
}
