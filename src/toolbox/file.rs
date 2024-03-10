
use reqwest;
use std::fs::{self};
use std::path::Path;
use std::io::{copy};

#[allow(dead_code)]
pub fn create_file_parent_directory(dest: &str) -> Result<(), String> {
    let path: &Path = Path::new(dest);
    if let Err(err) = std::fs::create_dir_all(path.parent().unwrap()) {
        return Err(err.to_string());
    }
    return Ok(());
}

pub async fn download_file_to(url: &str, dest : &str) -> Result<(), String> {
    let result =  reqwest::get(url).await;
    if let Err(err) = result {
        return Err(err.to_string());
    }
   
    // Download
    let response = result.unwrap();
    let bytes = response.bytes().await;
    let content = bytes.unwrap().as_ref().clone().to_vec();
    if let Err(err) = fs::write(Path::new(dest), &content) {
        return Err(err.to_string());
    }
    Ok(())
}

pub fn extract_zip(zip_file: &str, dir: &str) -> Result<(), String>{
    let zipfile = std::fs::File::open(zip_file);
    if let Err(err) = zipfile {
        return  Err(err.to_string());
    }
    let mut zip = zip::ZipArchive::new(zipfile.unwrap()).unwrap();

    let target = Path::new(dir);
    if !target.exists() {
        if let Err(err)= fs::create_dir_all(target) {
            return Err(err.to_string());
        }
    }
    for i in 0..zip.len() {
        let mut file = zip.by_index(i).unwrap();
        if file.is_dir() {
            println!("file utf8 path {:?}", file.name_raw()); //文件名编码,在windows下用winrar压缩的文件夹，中文文夹件会码(发现文件名是用操作系统本地编码编码的，我的电脑就是GBK),本例子中的压缩的文件再解压不会出现乱码
            let target = target.join(Path::new(&file.name().replace("\\", "")));
            if let Err(err) = fs::create_dir_all(target) {
                return Err(err.to_string());
            }
        } else {
            let file_path = target.join(Path::new(file.name()));
            let mut target_file = if !file_path.exists() {
                println!("file path {}", file_path.to_str().unwrap());
                fs::File::create(file_path).unwrap()
            } else {
                fs::File::open(file_path).unwrap()
            };
            if let Err(err) = copy(&mut file, &mut target_file) {
                return Err(err.to_string());
            }
            // target_file.write_all(file.read_bytes().into());
        }
    }
    Ok(())
}


pub async fn download_text(url: &str) -> Result<String, String> {
    let result =  reqwest::get(url).await;
    if let Err(err) = result {
        return Err(err.to_string());
    }

    match result {
        Ok(res) => Ok(res.text().await.unwrap()),
        Err(err) => Err(err.to_string()),
    }
}