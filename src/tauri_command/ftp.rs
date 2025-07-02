use std::sync::{Arc, Mutex};
use suppaftp::FtpStream;
use uuid::Uuid;

lazy_static::lazy_static! {
    static ref FTP_CLIENTS: Arc<Mutex<std::collections::HashMap<String, FtpStream>>> = 
        Arc::new(Mutex::new(std::collections::HashMap::new()));
}

#[tauri::command]
async fn connect_ftp(
    host: &str,
    port: u16,
    username: &str,
    password: &str,
) -> Result<String, String> {
    let mut ftp = FtpStream::connect(format!("{}:{}", host, port))
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