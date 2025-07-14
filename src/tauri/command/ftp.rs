use std::io::Write;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use suppaftp::FtpStream;
use tokio::io::AsyncReadExt;
use uuid::Uuid;
lazy_static::lazy_static! {
    static ref FTP_CLIENTS: Arc<Mutex<std::collections::HashMap<String, FtpStream>>> =
        Arc::new(Mutex::new(std::collections::HashMap::new()));
}

#[tauri::command]
pub async fn connect_ftp(
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
    let mut client = FTP_CLIENTS.lock().map_err(|_| "锁获取失败".to_string())?;
       let ftp = client
        .get_mut(key)
        .ok_or_else(|| "指定的FTP连接不存在".to_string())?;
    ftp.quit().await.map_err(|e| format!("断开连接失败: {}", e))?;
    client.remove(key);
    Ok(())
}

#[tauri::command]
pub async fn ftp_list(key: &str, path: &str) -> Result<Vec<String>, String> {
    let mut client = FTP_CLIENTS.lock().map_err(|_| "锁获取失败".to_string())?;
    let ftp = client
        .get_mut(key)
        .ok_or_else(|| "指定的FTP连接不存在".to_string())?;

    let files = ftp
        .list(Some(&format!("{}", path)))
        .map_err(|e| format!("获取文件列表失败: {}", e))?;
    Ok(files)
}

#[tauri::command]
pub async fn ftp_upload_file(key: &str, path: &str, local_file: &str) -> Result<(), String> {
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
    let mut client = FTP_CLIENTS.lock().map_err(|_| "锁获取失败".to_string())?;
    let ftp = client
        .get_mut(key)
        .ok_or_else(|| "指定的FTP连接不存在".to_string())?;

    let files = ftp
        .list(Some(&format!("{}", path)))
        .map_err(|e| format!("获取文件列表失败: {}", e))?;
    Ok(files)
}

#[tauri::command]
pub async fn ftp_upload_file(key: &str, path: &str, local_file: &str) -> Result<(), String> {
    let mut client = FTP_CLIENTS.lock().map_err(|_| "锁获取失败".to_string())?;
    let ftp = client
        .get_mut(key)
        .ok_or_else(|| "指定的FTP连接不存在".to_string())?;
    let mut reader = std::fs::File::open(local_file).map_err(|e| format!("打开文件失败: {}", e))?;
    ftp.put_file(&path, &mut reader)
        .map_err(|e| format!("上传文件失败: {}", e))?;
    Ok(())
}

#[tauri::command]
pub async fn ftp_download_file(key: &str, path: &str, local_file: &str) -> Result<(), String> {
    let mut client = FTP_CLIENTS.lock().map_err(|_| "锁获取失败".to_string())?;
    let ftp = client
        .get_mut(key)
        .ok_or_else(|| "指定的FTP连接不存在".to_string())?;
    let mut buf = ftp
        .retr_as_buffer(&path)
        .map_err(|e| format!("下载文件失败: {}", e))?;
    let mut data_buf = Vec::new();

    if std::path::Path::new(&local_file).exists() {
        let mut f = std::fs::File::open(local_file).map_err(|e| format!("打开文件失败: {}", e))?;
        if let Err(e) = buf.read_buf(&mut data_buf).await {
            return Err(format!("读取文件失败: {}", e));
        }
        f.write_all(&data_buf)
            .map_err(|e| format!("写入文件失败: {}", e))?;
    } else {
        let mut f =
            std::fs::File::create(local_file).map_err(|e| format!("创建文件失败: {}", e))?;
        if let Err(e) = buf.read_buf(&mut data_buf).await {
            return Err(format!("读取文件失败: {}", e));
        }

        f.write_all(&data_buf)
            .map_err(|e| format!("写入文件失败: {}", e))?;
    }
    Ok(())
}

#[tauri::command]
pub async fn ftp_delete_file(key: &str, path: &str) -> Result<(), String> {
    let mut client = FTP_CLIENTS.lock().map_err(|_| "锁获取失败".to_string())?;
    let ftp = client
        .get_mut(key)
        .ok_or_else(|| "指定的FTP连接不存在".to_string())?;
    ftp.rm(&format!("{}", path))
        .map_err(|e| format!("删除文件失败: {}", e))?;
    Ok(())
}

#[tauri::command]
pub async fn ftp_delete_dir(key: &str, path: &str) -> Result<(), String> {
    let mut client = FTP_CLIENTS.lock().map_err(|_| "锁获取失败".to_string())?;
    let ftp = client
        .get_mut(key)
        .ok_or_else(|| "指定的FTP连接不存在".to_string())?;
    ftp.rmdir(&format!("{}", path))
        .map_err(|e| format!("删除目录失败: {}", e))?;
    Ok(())
}
