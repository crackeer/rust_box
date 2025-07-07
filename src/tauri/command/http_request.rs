use super::define::{failure_response, success_response, InvokeResponse, Message};
use crate::toolbox::http_request;
use scraper::{Html, Selector};
use serde_json::json;
use serde_json::Value;
use std::collections::HashMap;
use tauri;

#[tauri::command]
pub fn do_http_request(
    url: String,
    method: String,
    data: Value,
    header: HashMap<String, String>,
) -> InvokeResponse {
    let client = reqwest::blocking::Client::new();
    let mut builder: reqwest::blocking::RequestBuilder;
    if method.to_lowercase() == "get" {
        builder = client.get(url).query(&data);
    } else {
        builder = client.post(&url).json(&data);
    }
    for (key, value) in header {
        builder = builder.header(String::from(key), String::from(value));
    }

    let response = builder.send();
    match response {
        Err(err) => InvokeResponse {
            success: false,
            message: err.to_string(),
            data: json!(null),
        },
        Ok(body) => {
            if body.status() != 200 {
                return InvokeResponse {
                    success: false,
                    message: format!("http status = {}", body.status()),
                    data: json!(null),
                };
            }
            let json_value: Value = body.json().unwrap();
            InvokeResponse {
                success: true,
                message: "success".to_string(),
                data: json_value,
            }
        }
    }
}

#[tauri::command]
pub async fn parse_js_code(url: String) -> Vec<String> {
    let mut list: Vec<String> = Vec::new();
    let result = http_request::download_text(&url).await;
    if let Err(_err) = result {
        return list;
    }
    let html = result.unwrap();

    let document = Html::parse_document(&html);
    let script_selector = Selector::parse("script").unwrap();

    for script in document.select(&script_selector) {
        let script_content = script.inner_html();
        list.push(script_content);
    }
    list
}

#[tauri::command]
pub async fn parse_html_title(url: String) -> String {
    let result = http_request::download_text(&url).await;
    if let Err(_err) = result {
        return String::from("");
    }
    let html = result.unwrap();

    let document = Html::parse_document(&html);
    let script_selector = Selector::parse("title").unwrap();

    for script in document.select(&script_selector) {
        return script.inner_html();
    }
    String::from("")
}

#[tauri::command]
pub async fn parse_github_ip() -> InvokeResponse {
    let result = http_request::download_text(&"https://sites.ipaddress.com/www.github.com/").await;
    if let Err(err) = result {
        return failure_response(Message::String(err.to_string()));
    }
    let html = result.unwrap();

    let document = Html::parse_document(&html);
    let tab_dns_selector = Selector::parse("#tabpanel-dns-a pre a").unwrap();

    for script in document.select(&tab_dns_selector) {
        let script_content = script.inner_html();
        return success_response(json!({
            "ip" : &script_content,
        }));
    }
    failure_response(Message::String(String::from("no ip found")))
}
