use super::define::InvokeResponse;
use serde::{Deserialize, Serialize};
use serde_json::json;
use ssh2::Session;
use std::fs;
use std::io::prelude::*;
use std::net::TcpStream;
use std::path::Path;

fn get_ssh_session(host: &str, private_key_path: &str) -> Result<Session, String> {
    let connection = TcpStream::connect(format!("{}:22", host));
    if let Err(err) = connection {
        return Err(err.to_string());
    }

    let mut session = Session::new().unwrap();
    session.set_tcp_stream(connection.unwrap());
    if let Err(err) = session.handshake() {
        return Err(format!("handshake error:{}", err.to_string()));
    }
    if let Err(err) = session.auth_methods(&"root") {
        return Err(format!("auth root error :{}", err.to_string()));
    }

    if let Err(err) = session.userauth_pubkey_file("root", None, Path::new(&private_key_path), None)
    {
        return Err(format!("userauth_pubkey_file error :{}", err.to_string()));
    }

    if !session.authenticated() {
        return Err(String::from("authenticated wrong"));
    }

    Ok(session)
}


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
pub async fn ls_files(host: String, private_key_path: String, path: String) -> InvokeResponse {
    let session = get_ssh_session(&host, &private_key_path);
    if let Err(err) = session {
        return InvokeResponse {
            success: false,
            message: err.to_string(),
            data: json!(null),
        };
    }
    let sess = session.unwrap();
    let mut channel = sess.channel_session().unwrap();
    channel.exec(&format!("ls -l {}", path)).unwrap();
    let mut result = String::new();
    _ = channel.read_to_string(&mut result);
    let list: Vec<&str> = result.split("\n").collect();
    let mut file_list: Vec<File> = Vec::new();
    for item in list.iter() {
        let parts: Vec<&str> = item.split(" ").filter(|x| x.len() > 0).collect();
        println!("{} - {}", parts.len(), parts.join("T").as_str());
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
    InvokeResponse {
        success: true,
        message: "".to_string(),
        data: json!(file_list),
    }
}

#[tauri::command]
pub async fn download_remote_file(
    host: String,
    private_key_path: String,
    path: String,
    local_save_path: String,
) -> InvokeResponse {
    let session = get_ssh_session(&host, &private_key_path);
    if let Err(err) = session {
        return InvokeResponse {
            success: false,
            message: err.to_string(),
            data: json!(null),
        };
    }
    let sess = session.unwrap();

    let (mut remote_file, stat) = sess.scp_recv(Path::new(&path.as_str())).unwrap();
    println!("remote file size: {}", stat.size());
    let mut contents = Vec::new();
    remote_file.read_to_end(&mut contents).unwrap();

    // Close the channel and wait for the whole content to be tranferred
    remote_file.send_eof().unwrap();
    remote_file.wait_eof().unwrap();
    remote_file.close().unwrap();
    remote_file.wait_close().unwrap();

    if let Err(err) = fs::write(local_save_path, &contents) {
        return InvokeResponse {
            success: false,
            message: err.to_string(),
            data: json!(null),
        };
    }
    return InvokeResponse {
        success: true,
        message: "success".to_string(),
        data: json!(null),
    };
}

#[tauri::command]
pub async fn upload_remote_file(
    host: String,
    private_key_path: String,
    path: String,
    local_file: String,
) -> InvokeResponse {
    let session = get_ssh_session(&host, &private_key_path);
    if let Err(err) = session {
        return InvokeResponse {
            success: false,
            message: err.to_string(),
            data: json!(null),
        };
    }
    let sess = session.unwrap();

    match fs::read(Path::new(&local_file.as_str())) {
        Ok(data) => {
            // Write the file
            let mut remote_file = sess
                .scp_send(Path::new(&path.as_str()), 0o644, 10, None)
                .unwrap();
            remote_file.write(&data).unwrap();
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
        Err(err) => {
            return InvokeResponse {
                success: false,
                message: err.to_string(),
                data: json!(null),
            }
        }
    }
}

#[tauri::command]
pub async fn remote_exec_cmd(host: String, private_key_path: String, cmd_string: String) -> InvokeResponse {
    let session = get_ssh_session(&host, &private_key_path);
    if let Err(err) = session {
        return InvokeResponse {
            success: false,
            message: err.to_string(),
            data: json!(null),
        };
    }
    let sess = session.unwrap();
    let mut channel = sess.channel_session().unwrap();
    channel.exec(&cmd_string).unwrap();
    let mut result = String::new();
    _ = channel.read_to_string(&mut result);
    InvokeResponse {
        success: true,
        message: "".to_string(),
        data: json!(result),
    }
}