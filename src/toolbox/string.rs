use md5::{Md5, Digest};

pub fn md5_string(input: &str) -> String {
    let hash = Md5::digest(input.as_bytes());
    hex::encode(hash) // 返回小写十六进制字符串
}