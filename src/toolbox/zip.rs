use std::fs::{self};
use std::io::copy;
use std::path::Path;

pub fn extract(zip_file: &str, dir: &str) -> Result<(), String> {
    let zipfile = std::fs::File::open(zip_file);
    if let Err(err) = zipfile {
        return Err(err.to_string());
    }
    let mut zip = zip::ZipArchive::new(zipfile.unwrap()).unwrap();

    let target = Path::new(dir);
    if !target.exists() {
        if let Err(err) = fs::create_dir_all(target) {
            return Err(err.to_string());
        }
    }
    for i in 0..zip.len() {
        let mut file = zip.by_index(i).unwrap();
        if file.is_dir() {
            println!("file utf8 path {:?}", file.name_raw());
            //文件名编码,在windows下用winrar压缩的文件夹，中文文夹件会码(发现文件名是用操作系统本地编码编码的，我的电脑就是GBK),本例子中的压缩的文件再解压不会出现乱码
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
        }
    }
    Ok(())
}
