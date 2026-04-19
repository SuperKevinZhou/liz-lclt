mod backend;

use std::sync::Arc;

use backend::commands::{
    cancel_translation, get_task_status, load_app_state, load_text_resource, save_blacklist,
    save_config, save_models, save_terminology, save_text_resource, save_translation_configs,
    set_workspace_root, start_translation,
};
use backend::state::AppState;
use tokio::sync::Mutex;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let state = Arc::new(Mutex::new(AppState::default()));

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            set_workspace_root,
            load_app_state,
            load_text_resource,
            save_config,
            save_models,
            save_translation_configs,
            save_blacklist,
            save_text_resource,
            save_terminology,
            start_translation,
            cancel_translation,
            get_task_status
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
