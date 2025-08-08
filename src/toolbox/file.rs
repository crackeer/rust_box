use serde::Serialize;
use std::{fs::read_dir, path::Path};
use readable::byte::Byte;
use base64::{engine::general_purpose, Engine as _};
use mime_guess;


#[allow(dead_code)]
pub fn create_file_parent_directory<P>(path : P) -> Result<(), String> 
where P: AsRef<str> {
    let path = Path::new(path.as_ref());
    let path = path.parent().ok_or("no parent")?;
    std::fs::create_dir_all(path).map_err(|err| err.to_string())?;
    return Ok(());
}

#[derive(Serialize)]
pub struct FileInfo {
    name: String,
    file_type: String,
    size : u64,
    human_size: String,
}

pub fn simple_read_dir(dir: String) -> Vec<FileInfo> {
    let mut list: Vec<FileInfo> = Vec::new();
    let entry = read_dir(dir);
    if entry.is_err() {
        return list;
    }
    for item in entry.unwrap().into_iter() {
        if let Err(_) = item {
            continue;
        }
        if let Ok(ref file) = item {
            let meta = file.metadata().unwrap();
            let mut name = String::from("");
            if let Ok(value) = file.file_name().into_string() {
                name = value;
            }
            if meta.is_dir() {
                list.push(FileInfo {
                    name: name,
                    file_type: String::from("directory"),
                    size: meta.len(),
                    human_size:Byte::from(meta.len()).to_string(),
                });
            } else {
                list.push(FileInfo {
                    name: name,
                    file_type: String::from("file"),
                    size : meta.len(),
                    human_size:Byte::from(meta.len()).to_string(),
                });
            }
        }
    }
    list
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_simple_read_dir() {
        let result = simple_read_dir(String::from("/Users/liuhu016/rust/rust_box"));
        for item in result.iter() {
            println!("{}-{}-{}", item.name, item.file_type, item.human_size)
        }
    }
}

pub fn create_jsonp_file(src_file: &str, dest_file: &str, hash_code: &str) -> Result<(), String> {
    let content = std::fs::read(src_file).map_err(|e| e.to_string())?;
    let content_type = mime_guess::from_path(src_file).first().map_or(String::from(""), |v| v.to_string());
    _ = create_file_parent_directory(dest_file).map_err(|e| e);
    let jsonp_content = generate_jsonp_content(content_type.as_str(), &content, &hash_code);
    std::fs::write(dest_file, jsonp_content).unwrap();
    Ok(())
}

pub fn generate_jsonp_content(content_type: &str, input: &[u8], hash_code: &str) -> String {
    match content_type {
        "image/jpeg" | "image/jpg" => format!(
            "window[\"jsonp_{}\"] && window[\"jsonp_{}\"](\"data:image/jpeg;base64,{}\")",
            hash_code,
            hash_code,
            general_purpose::STANDARD.encode(input)
        ),
        "image/png" => format!(
            "window[\"jsonp_{}\"] && window[\"jsonp_{}\"](\"data:image/png;base64,{}\")",
            hash_code,
            hash_code,
            general_purpose::STANDARD.encode(input)
        ),
        _ => format!(
            "window['jsonp_{}'] && window['jsonp_{}'](\"data:application/octet-stream;base64,{}\")",
            hash_code,
            hash_code,
            general_purpose::STANDARD.encode(input)
        ),
    }
}