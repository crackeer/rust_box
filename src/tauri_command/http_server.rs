use super::define::InvokeResponse;
use axum::Router;
use serde_json::json;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio;
use tokio::time::interval;
use tower_http::services::ServeDir;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref SERVER_RUNNING: Arc<Mutex<usize>> = Arc::new(Mutex::new(0));
    static ref SERVER_CONFIG: Arc<Mutex<(String, u16, usize)>> =
        Arc::new(Mutex::new((String::new(), 0, 0)));
}

fn set_server_config(static_path: String, port: u16, running: usize) {
    if let Ok(mut config) = SERVER_CONFIG.lock() {
        config.0 = static_path;
        config.1 = port;
        config.2 = running
    }
}

fn unset_server_config() {
    set_server_config(String::new(), 0, 0)
}

fn get_server_config() -> (String, u16, usize) {
    if let Ok(config) = SERVER_CONFIG.lock() {
        return (config.0.clone(), config.1, config.2);
    }
    return (String::new(), 0, 0);
}

fn is_server_running() -> bool {
    let (_, _, running) = get_server_config();
    return running > 0;
}


#[tauri::command]
pub async fn start_static_server(static_path: String, port: u16) -> InvokeResponse {
    if is_server_running() {
        return InvokeResponse {
            success: false,
            message: "a web server is running".to_string(),
            data: json!(null),
        };
    }
    //let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    let router = Router::new().nest_service("/", ServeDir::new(static_path.clone()));

    tokio::spawn(async move {
        axum::serve(listener, router)
            .with_graceful_shutdown(shutdown_signal())
            .await
            .unwrap();
        println!("Stopping web server");
    });
    set_server_config(static_path.clone(), port, 1);
    InvokeResponse {
        success: true,
        message: "success".to_string(),
        data: json!(null),
    }
}

async fn shutdown_signal() {
    println!("shutdown_signal installed");
    let mut interval = interval(Duration::from_secs(1));
    loop {
        interval.tick().await;
        println!("shutdown_signal loop 1s");
        if !is_server_running() {
            break;
        }
    }
    println!("shutdown_signal signal stop");
}


#[tauri::command]
pub async fn stop_static_server() -> InvokeResponse {
    if !is_server_running() {
        return InvokeResponse {
            success: false,
            message: "web server not running".to_string(),
            data: json!(null),
        };
    }
    unset_server_config();
    InvokeResponse {
        success: true,
        message: "success".to_string(),
        data: json!(null),
    }
}

#[tauri::command]
pub async fn static_server_status() -> InvokeResponse {
    let (path, port, status) = get_server_config();
    InvokeResponse {
        success: true,
        message: "success".to_string(),
        data: json!({
            "running" : status,
            "staticPath" : path,
            "port" : port,
        }),
    }
}
