use std::fs;
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "static/rsvrjsonp"]
struct RsvrJsonpAsset;

#[tauri::command]
pub async fn write_rsvr_jsonp_asset(dir: String) -> Result<(), String> {
     let preview_path = std::path::Path::new(&dir);
     for f in RsvrJsonpAsset::iter() {
        let a = RsvrJsonpAsset::get(f.as_ref()).unwrap();
        if let Err(err) = fs::write(preview_path.join(f.as_ref()), a.data.as_ref()) {
            return Err(err.to_string());
        }
    }
    Ok(())
}
