
use reqwest;
use std::fs::{self};
use std::path::Path;


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