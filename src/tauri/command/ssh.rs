use super::define::InvokeResponse;
use rand::distributions::{Alphanumeric, DistString};
use serde::{Deserialize, Serialize};
use serde_json::json;
use ssh2::DisconnectCode::AuthCancelledByUser;
use ssh2::Session;
use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::fs;
use std::io::prelude::*;
use std::io::{BufRead, BufReader};
use std::net::TcpStream;
use std::path::Path;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TransferInfo {
    local_file: String,
    remote_file: String,
    total: u64,
    current: u64,
    status: String,
    message: String,
}

lazy_static! {
    pub static ref SESSION_MAP: Arc<Mutex<HashMap<String, Session>>> =
        Arc::new(Mutex::new(HashMap::new()));
    pub static ref UPLOAD_PROGRESS: Arc<Mutex<(u64, u64)>> = Arc::new(Mutex::new((100, 0)));
    pub static ref TRANSFER_INFO: Arc<Mutex<TransferInfo>> = Arc::new(Mutex::new(TransferInfo {
        local_file: String::new(),
        remote_file: String::new(),
        total: 0,
        current: 0,
        status: String::new(),
        message: String::new(),
    }));
}

fn init_transfer_info(local_file: String, remote_file: String, total_size: u64) {
    let mut transfer_info = TRANSFER_INFO.lock().unwrap();
    let transfer_info = transfer_info.borrow_mut();
    transfer_info.local_file = local_file;
    transfer_info.remote_file = remote_file;
    transfer_info.total = total_size;
    transfer_info.current = 0;
    transfer_info.status = String::from("transferring");
    transfer_info.message = String::from("");
}

fn incr_transfer_size(size: u64) {
    let mut transfer_info = TRANSFER_INFO.lock().unwrap();
    let transfer_info = transfer_info.borrow_mut();
    transfer_info.current = transfer_info.current + size;
}


fn mark_transfer_failure(message: String) {
    let mut transfer_info = TRANSFER_INFO.lock().unwrap();
    let transfer_info = transfer_info.borrow_mut();
    transfer_info.status = String::from("failure");
    transfer_info.message = message;
}

fn mark_transfer_success() {
    let mut transfer_info = TRANSFER_INFO.lock().unwrap();
    let transfer_info = transfer_info.borrow_mut();
    transfer_info.status = String::from("success");
    transfer_info.current = transfer_info.total;
    transfer_info.message = String::from("");
}

static mut CANCEL_SIGNAL: i32 = 10;
const BUFFER_SIZE: usize = 1024;
const AUTH_TYPE_PASSWORD: &str = &"password";

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct File {
    access: String,
    group: String,
    user: String,
    size: String,
    month: String,
    day: String,
    time: String,
    name: String,
    is_dir: bool,
}

/*
-rw-------. 1 roo roo 2897 Jun 16 15:20 anaconda-ks.cfg
drwxrwxr-x 13 501 games 4096 Jul 20 15:54 download
-rw-r--r-- 1 roo roo 1024507392 Jul 20 15:57 download. ar
-rw-r--r--. 1 roo roo 2723 Jun 16 15:19 ini .log
-rw-r--r--. 1 roo roo 16108 Jun 16 15:20 ks-pos .log
-rw-------. 1 roo roo 2832 Jun 16 15:20 original-ks.cfg
drwx------ 4 501 games 4096 Jan 29 17:17 projec -src
-rw-r--r-- 1 roo roo 48936960 Jul 7 14:53 realsee-open-admin. ar
-rw-r--r-- 1 roo roo 77155328 Jul 6 18:20 realsee-open-svc. ar
-rw-r--r-- 1 roo roo 67422720 Jul 7 15:23 realsee-shepherd-alg. ar
-rw-r--r-- 1 roo roo 67352576 Jun 27 17:59 realsee-shepherd-svc. ar
-rw-r--r-- 1 roo roo 706563 Jun 19 14:33 realsee-vr-local.2.3.0-5cf4997-nuc. ar.gz
*/
#[tauri::command]
pub async fn remote_list_files(session_key: String, path: String) -> Result<Vec<File>, String> {
    let list = SESSION_MAP
        .lock()
        .map_err(|e| format!("get session map error:{}", e))?;
    let session = list.get(&session_key).ok_or("no session")?;
    let mut channel = session
        .channel_session()
        .map_err(|e| format!("get channel error:{}", e))?;
    channel
        .exec(&format!("ls -l {}", path))
        .map_err(|e| format!("exec error:{}", e))?;
    let mut result = String::new();
    _ = channel
        .read_to_string(&mut result)
        .map_err(|e| format!("read error:{}", e))?;
    let list: Vec<&str> = result.split("\n").collect();
    let mut file_list: Vec<File> = Vec::new();
    for item in list.iter() {
        let parts: Vec<&str> = item.split(" ").filter(|x| x.len() > 0).collect();
        if parts.len() < 9 {
            continue;
        }
        file_list.push(File {
            access: String::from(parts[0]),
            group: String::from(parts[2]),
            user: String::from(parts[3]),
            size: String::from(parts[4]),
            month: String::from(parts[5]),
            day: String::from(parts[6]),
            time: String::from(parts[7]),
            name: String::from(parts[8]),
            is_dir: parts[0].starts_with("d"),
        })
    }
    Ok(file_list)
}

#[tauri::command]
pub async fn download_remote_file(
    session_key: String,
    local_file: String,
    remote_file: String,
) -> Result<String, String> {
    let list = SESSION_MAP
        .lock()
        .map_err(|e| format!("get session map error:{}", e))?;
    let session = list.get(&session_key).ok_or("no session")?;

    // 1. get remote file size
    let (mut remote_channel, stat) = session
        .scp_recv(Path::new(&remote_file.as_str()))
        .map_err(|e| format!("scp recv error:{}", e))?;
    let mut buffer = [0u8; BUFFER_SIZE];
    let file_path = Path::new(&local_file);
    // 2. create dir if not exists
    let save_dir = file_path.parent().ok_or("no parent dir")?;
    fs::create_dir_all(save_dir).map_err(|e| format!("create dir error:{}", e))?;
    let mut tmp_file =
        fs::File::create(file_path).map_err(|e| format!("create file error:{}", e))?;

    tokio::spawn(async move {
        let mut download_size = 0;
        init_transfer_info(local_file, remote_file, stat.size() as u64);
        loop {
            unsafe {
                if CANCEL_SIGNAL > 0 {
                    break;
                }
            }
            match remote_channel.read(buffer.as_mut_slice()) {
                Ok(size) => {
                    download_size += size;
                    let _ = tmp_file.write(&buffer[..size]);
                    incr_transfer_size(size as u64);
                }
                Err(err) => {
                    mark_transfer_failure(err.to_string());
                    break;
                }
            }
        }
        mark_transfer_success();
        remote_channel.send_eof().unwrap();
        remote_channel.wait_eof().unwrap();
        remote_channel.close().unwrap();
        remote_channel.wait_close().unwrap();
    });
    Ok(String::from("success"))
}

#[tauri::command]
pub async fn upload_remote_file(
    session_key: String,
    local_file: String,
    remote_file: String,
) -> Result<String, String> {
    let list = SESSION_MAP.lock().unwrap();
    let session = list.get(&session_key).ok_or("no session")?;

    let metadata = fs::metadata(&local_file.to_string()).map_err(|e| e.to_string())?;
    if metadata.len() < 1 {
        mark_transfer_failure(String::from("file empty"));
        return Err(String::from("file empty"));
    }

    let mut remote_channel = session
        .scp_send(Path::new(&remote_file.as_str()), 0o644, metadata.len(), None)
        .map_err(|e| e.to_string())?;
    let tmp_file = fs::File::open(Path::new(&local_file.as_str())).map_err(|e| e.to_string())?;
     unsafe { CANCEL_SIGNAL = 0 }
    tokio::spawn(async move  {
        let mut reader = BufReader::new(tmp_file); // 创建 BufReader
        init_transfer_info(local_file.to_string(), remote_file.to_string(), metadata.len());
        loop {
            unsafe {
                if CANCEL_SIGNAL > 0 {
                    break;
                }
            }
            let result = reader.fill_buf();
            if let Err(err) = result {
                mark_transfer_failure(err.to_string());
                break;
            }
            let size = result.unwrap().len();
            println!("upload size: {}", size);
            if size > 0 {
                if let Err(err) = remote_channel.write(reader.buffer()) {
                    mark_transfer_failure(err.to_string());
                    break;
                }
            } else {
                break;
            }
            reader.consume(size);
            incr_transfer_size(size as u64);
        }
        mark_transfer_success();
        remote_channel.send_eof().unwrap();
        remote_channel.wait_eof().unwrap();
        remote_channel.close().unwrap();
        remote_channel.wait_close().unwrap();
    });

    Ok(String::from("success"))
}

fn connect_ssh_session(
    user: &str,
    host: &str,
    port: &str,
    auth_type: &str,
    auth_config: &str,
) -> Result<Session, String> {
    let connection = TcpStream::connect(format!("{}:{}", host, port));
    if let Err(err) = connection {
        return Err(err.to_string());
    }

    let mut session = Session::new().unwrap();
    session.set_tcp_stream(connection.unwrap());
    if let Err(err) = session.handshake() {
        return Err(format!("handshake error:{}", err.to_string()));
    }
    if let Err(err) = session.auth_methods(user) {
        return Err(format!("auth root error :{}", err.to_string()));
    }

    if AUTH_TYPE_PASSWORD.eq(auth_type) {
        if let Err(err) = session.userauth_password(user, auth_config) {
            return Err(format!(
                "userauth_password error :{},{},{}",
                err.to_string(),
                user,
                auth_config
            ));
        }
    } else if let Err(err) = session.userauth_pubkey_file(user, None, Path::new(&auth_config), None)
    {
        return Err(format!("userauth_pubkey_file error :{}", err.to_string()));
    }

    if !session.authenticated() {
        return Err(String::from("authenticated wrong"));
    }

    Ok(session)
}

#[tauri::command]
pub async fn test_server_connect(
    user: String,
    host: String,
    port: String,
    auth_type: String,
    auth_config: String,
) -> InvokeResponse {
    let session = connect_ssh_session(&user, &host, &port, &auth_type, &auth_config);   
    if let Err(err) = session {
        return InvokeResponse {
            success: false,
            message: err.to_string(),
            data: json!(null),
        };
    }
    return InvokeResponse {
        success: true,
        message: "ok".to_string(),
        data: json!(null),
    };
}

#[tauri::command]
pub async fn connect_server(
    user: String,
    host: String,
    port: String,
    auth_type: String,
    auth_config: String,
) -> Result<String, String> {
    let session = connect_ssh_session(&user, &host, &port, &auth_type, &auth_config).map_err(|e| e.to_string())?;
    let session_key = Alphanumeric.sample_string(&mut rand::thread_rng(), 16);
    SESSION_MAP
        .lock()
        .unwrap()
        .insert(session_key.clone(), session);
    Ok(session_key)
}

#[tauri::command]
pub async fn remote_exec_command(
    session_key: String,
    cmd_string: String,
    split: bool,
) -> InvokeResponse {
    let list = SESSION_MAP.lock().unwrap();
    match list.get(&session_key) {
        Some(sess) => {
            let mut channel = sess.channel_session().unwrap();
            channel.exec(&cmd_string).unwrap();
            let mut result = String::new();
            _ = channel.read_to_string(&mut result);
            if split {
                let list: Vec<&str> = result.split("\n").collect();
                return InvokeResponse {
                    success: true,
                    message: "".to_string(),
                    data: json!(list),
                };
            }

            return InvokeResponse {
                success: true,
                message: "".to_string(),
                data: serde_json::Value::String(result),
            };
        }
        None => InvokeResponse {
            success: false,
            message: String::from("no session"),
            data: json!(null),
        },
    }
}

#[tauri::command]
pub async fn get_transfer_remote_progress() -> Result<TransferInfo, String> {
    let progress = TRANSFER_INFO.lock().map_err(|e| e.to_string())?;
    Ok(progress.clone())
}

#[tauri::command]
pub async fn send_cancel_signal() -> InvokeResponse {
    unsafe { CANCEL_SIGNAL = 1 }
    InvokeResponse {
        success: true,
        message: String::from("ok"),
        data: json!(null),
    }
}

#[tauri::command]
pub async fn disconnect_server(session_key: String) -> InvokeResponse {
    let mut session_map = SESSION_MAP.lock();

    match session_map.as_mut() {
        Ok(list) => match list.get(&session_key) {
            Some(sess) => {
                if let Err(e) = sess.disconnect(Some(AuthCancelledByUser), &"user action", None) {
                    return InvokeResponse {
                        success: true,
                        message: String::from(e.message()),
                        data: json!(null),
                    };
                }
                list.remove(&session_key);
                InvokeResponse {
                    success: true,
                    message: String::from("success"),
                    data: json!(null),
                }
            }
            None => InvokeResponse {
                success: true,
                message: String::from("no session found"),
                data: json!(null),
            },
        },
        Err(err) => InvokeResponse {
            success: true,
            message: err.to_string(),
            data: json!(null),
        },
    }
}
