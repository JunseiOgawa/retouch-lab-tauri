mod retouch;

#[tauri::command]
fn list_strategies() -> Vec<retouch::RetouchStrategyDefinition> {
    retouch::list_strategies()
}

#[tauri::command]
fn apply_retouch(
    request: retouch::ApplyRetouchRequest,
) -> Result<retouch::ApplyRetouchResponse, String> {
    retouch::apply_retouch(request)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![list_strategies, apply_retouch])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
