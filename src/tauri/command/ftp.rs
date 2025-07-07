use std::sync::{Arc, Mutex};
use suppaftp::FtpStream;
use uuid::Uuid;
use std::net::{ SocketAddr};
use std::time::Duration;

lazy_static::lazy_static! {
    static ref FTP_CLIENTS: Arc<Mutex<std::collections::HashMap<String, FtpStream>>> = 
        Arc::new(Mutex::new(std::collections::HashMap::new()));
}

#[tauri::command]
pub  async  fn connect_ftp(
    host: &str,
    port: u16,
    username: &str,
    password: &str,
) -> Result<String, String> {
    let url = format!("{}:{}", host, port);
    let addr: SocketAddr = url.parse().expect("invalid hostname");
    let mut ftp = FtpStream::connect_timeout(addr, Duration::from_secs(10))
        .map_err(|e| format!("连接失败: {}", e))?;
    
    ftp.login(username, password)
        .map_err(|e| format!("认证失败: {}", e))?;

    let key = Uuid::new_v4().to_string();
    FTP_CLIENTS
        .lock()
        .map_err(|_| "锁获取失败".to_string())?
        .insert(key.clone(), ftp);
    Ok(key)
}

#[tauri::command]
pub async fn disconnect_ftp(key: &str) -> Result<(), String> {
    FTP_CLIENTS
        .lock()
        .map_err(|_| "锁获取失败".to_string())?
        .remove(key)
        .ok_or_else(|| "连接不存在".to_string())?
        .quit()
        .map_err(|e| format!("断开连接失败: {}", e))?;
    Ok(())
}

#[tauri::command]
pub async fn ftp_list(key: &str, path: &str) -> Result<Vec<String>, String> {
    let  mut client = FTP_CLIENTS.lock().map_err(|_| "锁获取失败".to_string())?;
    let ftp =  client.get_mut(key).ok_or_else(|| "指定的FTP连接不存在".to_string())?;

    let files = ftp.list(Some(&format!("{}", path))).map_err(|e| format!("获取文件列表失败: {}", e))?;
    Ok(files)
}