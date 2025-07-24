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
pub struct UploadInfo {
    local_file: String,
    remote_file: String,
    total_size: u64,
    upload_size: u64,
    status: String,
    message: String,
}

lazy_static! {
    pub static ref SESSION_MAP: Arc<Mutex<HashMap<String, Session>>> =
        Arc::new(Mutex::new(HashMap::new()));
    pub static ref UPLOAD_PROGRESS: Arc<Mutex<(u64, u64)>> = Arc::new(Mutex::new((100, 0)));
    pub static ref UPLOAD_INFO: Arc<Mutex<UploadInfo>> = Arc::new(Mutex::new(UploadInfo {
        local_file: String::new(),
        remote_file: String::new(),
        total_size: 0,
        upload_size: 0,
        status: String::new(),
        message: String::new(),
    }));
}

fn init_upload_info(local_file: String, remote_file: String, total_size: u64) {
    let mut upload_info = UPLOAD_INFO.lock().unwrap();
    let upload_info = upload_info.borrow_mut();
    upload_info.local_file = local_file;
    upload_info.remote_file = remote_file;
    upload_info.total_size = total_size;
    upload_info.upload_size = 0;
    upload_info.status = String::from("running");
    upload_info.message = String::from("");
}

fn incr_upload_size(size: u64) {
    let mut upload_info = UPLOAD_INFO.lock().unwrap();
    let upload_info = upload_info.borrow_mut();
    upload_info.upload_size = upload_info.upload_size + size;
}

fn clear_upload_info() {
    let mut upload_info = UPLOAD_INFO.lock().unwrap();
    let upload_info = upload_info.borrow_mut();
    upload_info.local_file = String::from("");
    upload_info.remote_file = String::from("");
}

fn mark_upload_failure(message: String) {
    let mut upload_info = UPLOAD_INFO.lock().unwrap();
    let upload_info = upload_info.borrow_mut();
    upload_info.status = String::from("failure");
    upload_info.message = message;
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
    app: AppHandle,
    session_key: String,
    path: String,
    local_save_path: String,
) -> Result<serde_json::Value, String> {
    let list = SESSION_MAP
        .lock()
        .map_err(|e| format!("get session map error:{}", e))?;
    let session = list.get(&session_key).ok_or("no session")?;

    // 1. get remote file size
    let (mut remote_file, stat) = session
        .scp_recv(Path::new(&path.as_str()))
        .map_err(|e| format!("scp recv error:{}", e))?;
    println!("remote file size: {}", stat.size());
    let mut buffer = [0u8; BUFFER_SIZE];
    let file_path = Path::new(&local_save_path);
    let mut tmp_file  = fs::File::create(file_path).map_err(|e| format!("create file error:{}", e))?;
    // 2. create dir if not exists
    let save_dir = file_path.parent().ok_or("no parent dir")?;
    fs::create_dir_all(save_dir).map_err(|e| format!("create dir error:{}", e))?;
    tokio::spawn(async move {
        let mut download_size = 0;
        loop {
            unsafe {
                if CANCEL_SIGNAL > 0 {
                    break;
                }
            }
            match remote_file.read(buffer.as_mut_slice()) {
                Ok(size) => {
                    download_size += size;
                    let _ = tmp_file.write(&buffer[..size]);
                    app.emit("download-progress", download_size).unwrap();
                }
                Err(err) => {
                    println!("download error:{}", err);
                    break;
                }
            }
        }
        remote_file.send_eof().unwrap();
        remote_file.wait_eof().unwrap();
        remote_file.close().unwrap();
        remote_file.wait_close().unwrap();
    });
    Ok(json!({
        "path": local_save_path,
        "size": stat.size(),
    }))
}

#[tauri::command]
pub async fn upload_remote_file(
    session_key: String,
    path: String,
    local_file: String,
) -> InvokeResponse {
    let list = SESSION_MAP.lock().unwrap();
    let session = list.get(&session_key);
    if let None = session {
        return InvokeResponse {
            success: false,
            message: String::from("no session"),
            data: json!(null),
        };
    }
    let sess = session.unwrap();
    let file_size = match fs::metadata(&local_file.to_string()) {
        Ok(meta) => meta.len(),
        Err(_) => 0,
    };
    if file_size < 1 {
        mark_upload_failure(String::from("file empty"));
        return InvokeResponse {
            success: false,
            message: String::from("file empty"),
            data: json!(null),
        };
    }

    let mut remote_file = sess
        .scp_send(Path::new(&path.as_str()), 0o644, file_size, None)
        .unwrap();

    let tmp_file = fs::File::open(Path::new(&local_file.as_str()));

    let mut reader = BufReader::new(tmp_file.unwrap()); // 创建 BufReader
    init_upload_info(local_file.to_string(), path.to_string(), file_size);
    unsafe { CANCEL_SIGNAL = 0 }
    loop {
        unsafe {
            if CANCEL_SIGNAL > 0 {
                return InvokeResponse {
                    success: false,
                    message: String::from("user cancelled"),
                    data: json!(null),
                };
            }
        }

        let result = reader.fill_buf();
        if let Err(err) = result {
            mark_upload_failure(err.to_string());
            return InvokeResponse {
                success: false,
                message: err.to_string(),
                data: json!(null),
            };
        }
        let size = result.unwrap().len();
        if size > 0 {
            if let Err(err) = remote_file.write(reader.buffer()) {
                mark_upload_failure(err.to_string());
                return InvokeResponse {
                    success: false,
                    message: err.to_string(),
                    data: json!(null),
                };
            }
        } else {
            break;
        }
        reader.consume(size);
        incr_upload_size(size as u64);
    }
    clear_upload_info();
    remote_file.send_eof().unwrap();
    remote_file.wait_eof().unwrap();
    remote_file.close().unwrap();
    remote_file.wait_close().unwrap();

    InvokeResponse {
        success: true,
        message: "success".to_string(),
        data: json!(null),
    }
}

fn connect_ssh_session(
    user: &str,
    host: &str,
    auth_type: &str,
    auth_config: &str,
) -> Result<Session, String> {
    let connection = TcpStream::connect(format!("{}:22", host));
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
    auth_type: String,
    auth_config: String,
) -> InvokeResponse {
    let session = connect_ssh_session(&user, &host, &auth_type, &auth_config);
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
    auth_type: String,
    auth_config: String,
) -> InvokeResponse {
    let session = connect_ssh_session(&user, &host, &auth_type, &auth_config);
    if let Err(err) = session {
        return InvokeResponse {
            success: false,
            message: err.to_string(),
            data: json!(null),
        };
    }
    let session_key = Alphanumeric.sample_string(&mut rand::thread_rng(), 16);
    SESSION_MAP
        .lock()
        .unwrap()
        .insert(session_key.clone(), session.unwrap());
    return InvokeResponse {
        success: true,
        message: "ok".to_string(),
        data: json!({ "session_key": session_key }),
    };
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
pub async fn get_upload_progress() -> InvokeResponse {
    let progress = UPLOAD_INFO.lock().unwrap();
    InvokeResponse {
        success: true,
        message: String::from("ok"),
        data: json!(progress.clone()),
    }
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
