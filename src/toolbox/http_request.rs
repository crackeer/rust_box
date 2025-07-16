
use reqwest;
use std::fs::{self};
use std::path::Path;


pub async fn download_file_to(url: &str, dest : &str) ->  Result<(), String> {
    match reqwest::get(url).await  {
        Ok(res) => {
            let bytes = res.bytes().await;
            let content = bytes.unwrap().as_ref().to_vec();
            if let Err(err) = fs::write(Path::new(dest), &content) {
                return Err(err.to_string());
            }
            Ok(())
        }
        Err(err) => Err(err.to_string())
    }
}

pub async fn download_file_to_v2(url: &str, dest : &str) -> Result<String, String> {
    match reqwest::get(url).await  {
        Ok(res) => {
            let bytes = res.bytes().await;
            let content = bytes.unwrap().as_ref().to_vec();
            if let Err(err) = fs::write(Path::new(dest), &content) {
                return Err(err.to_string());
            }
            Ok(String::from_utf8_lossy(&content).to_string())
        }
        Err(err) => Err(err.to_string())
    }
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