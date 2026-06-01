pub mod parser;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
async fn convert_to_markdown(file_path: String) -> Result<String, String> {
    tokio::task::spawn_blocking(move || {
        parser::parse_file(&file_path)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![convert_to_markdown])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
