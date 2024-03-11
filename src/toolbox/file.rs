use serde::Serialize;
use std::{fs::read_dir, path::Path};
use readable::byte::Byte;

#[allow(dead_code)]
pub fn create_file_parent_directory(dest: &str) -> Result<(), String> {
    let path: &Path = Path::new(dest);
    if let Err(err) = std::fs::create_dir_all(path.parent().unwrap()) {
        return Err(err.to_string());
    }
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
