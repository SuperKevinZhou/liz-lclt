use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use tauri::path::BaseDirectory;
use tauri::Manager;
use tokio::sync::Mutex;

use crate::backend::config::{
    default_workspace_paths, display_path, load_payload, resolve_existing_resource_path,
    resolve_resource_path_for_write, save_app_config, save_blacklist as persist_blacklist,
    save_models as persist_models, save_translation_configs as persist_translation_configs,
    strip_windows_extended_prefix,
};
use crate::backend::error::UserFacingError;
use crate::backend::models::{
    AppConfig, AppStatePayload, BlacklistConfig, ModelsConfig, SaveTerminologyPayload,
    SaveTextResourcePayload, TextResourcePayload, TranslationConfigs, TranslationRunOptions,
    TranslationTask, WorkspaceSelection,
};
use crate::backend::resources::{read_text_file, write_terminology, write_text_file};
use crate::backend::state::AppState;
use crate::backend::task::start_translation_task;

fn resolve_workspace_root(
    app: Option<&tauri::AppHandle>,
    state: &AppState,
    explicit: Option<String>,
) -> Result<PathBuf, UserFacingError> {
    if let Some(path) = explicit {
        return Ok(PathBuf::from(strip_windows_extended_prefix(&path)));
    }

    if let Some(path) = &state.workspace_root {
        return Ok(path.clone());
    }

    if let Some(app) = app {
        if let Ok(dir) = app.path().resolve("workspace", BaseDirectory::AppLocalData) {
            fs::create_dir_all(&dir)
                .map_err(|error| UserFacingError::io("Create directory", &dir, &error))?;
            return Ok(dir);
        }
    }

    std::env::current_dir().map_err(|error| {
        UserFacingError::new(
            "Workspace Required",
            "Failed to resolve the default workspace directory.",
            Some(error.to_string()),
        )
    })
}

#[tauri::command]
pub async fn set_workspace_root(
    state: tauri::State<'_, Arc<Mutex<AppState>>>,
    payload: WorkspaceSelection,
) -> Result<String, UserFacingError> {
    let mut guard = state.lock().await;
    let workspace_root = PathBuf::from(strip_windows_extended_prefix(&payload.workspace_root));
    guard.workspace_root = Some(workspace_root.clone());
    Ok(display_path(&workspace_root))
}

#[tauri::command]
pub async fn load_app_state(
    app: tauri::AppHandle,
    state: tauri::State<'_, Arc<Mutex<AppState>>>,
    workspace_root: Option<String>,
) -> Result<AppStatePayload, UserFacingError> {
    let root = {
        let mut guard = state.lock().await;
        let root = resolve_workspace_root(Some(&app), &guard, workspace_root)?;
        guard.workspace_root = Some(root.clone());
        root
    };

    let mut payload = load_payload(root);
    let guard = state.lock().await;
    if let Some(task) = &guard.task {
        payload.current_task = Some(task.task.clone());
    }
    Ok(payload)
}

#[tauri::command]
pub async fn load_text_resource(
    state: tauri::State<'_, Arc<Mutex<AppState>>>,
    payload: TextResourcePayload,
) -> Result<TextResourcePayload, UserFacingError> {
    let guard = state.lock().await;
    let workspace_root = resolve_workspace_root(None, &guard, None)?;
    let relative_path = payload.path.clone();
    let path =
        resolve_existing_resource_path(&workspace_root, &relative_path).ok_or_else(|| {
            UserFacingError::new(
                "Missing Resource",
                format!("Could not locate '{}'.", relative_path),
                None,
            )
        })?;
    let content = read_text_file(&path)?;
    Ok(TextResourcePayload {
        path: relative_path,
        content,
    })
}

#[tauri::command]
pub async fn save_config(
    state: tauri::State<'_, Arc<Mutex<AppState>>>,
    payload: AppConfig,
) -> Result<(), UserFacingError> {
    let workspace_root = {
        let guard = state.lock().await;
        resolve_workspace_root(None, &guard, None)?
    };
    let paths = default_workspace_paths(workspace_root);
    save_app_config(&paths.config, &payload)
}

#[tauri::command]
pub async fn save_models(
    state: tauri::State<'_, Arc<Mutex<AppState>>>,
    payload: ModelsConfig,
) -> Result<(), UserFacingError> {
    let workspace_root = {
        let guard = state.lock().await;
        resolve_workspace_root(None, &guard, None)?
    };
    let paths = default_workspace_paths(workspace_root);
    persist_models(&paths.models, &payload)
}

#[tauri::command]
pub async fn save_translation_configs(
    state: tauri::State<'_, Arc<Mutex<AppState>>>,
    payload: TranslationConfigs,
) -> Result<(), UserFacingError> {
    let workspace_root = {
        let guard = state.lock().await;
        resolve_workspace_root(None, &guard, None)?
    };
    let paths = default_workspace_paths(workspace_root);
    persist_translation_configs(&paths.translation_configs, &payload)
}

#[tauri::command]
pub async fn save_blacklist(
    state: tauri::State<'_, Arc<Mutex<AppState>>>,
    payload: BlacklistConfig,
) -> Result<(), UserFacingError> {
    let workspace_root = {
        let guard = state.lock().await;
        resolve_workspace_root(None, &guard, None)?
    };
    let paths = default_workspace_paths(workspace_root);
    persist_blacklist(&paths.blacklist, &payload)
}

#[tauri::command]
pub async fn save_text_resource(
    state: tauri::State<'_, Arc<Mutex<AppState>>>,
    payload: SaveTextResourcePayload,
) -> Result<(), UserFacingError> {
    let workspace_root = {
        let guard = state.lock().await;
        resolve_workspace_root(None, &guard, None)?
    };
    let path = resolve_resource_path_for_write(&workspace_root, &payload.path);
    write_text_file(&path, &payload.content)
}

#[tauri::command]
pub async fn save_terminology(
    state: tauri::State<'_, Arc<Mutex<AppState>>>,
    payload: SaveTerminologyPayload,
) -> Result<(), UserFacingError> {
    let workspace_root = {
        let guard = state.lock().await;
        resolve_workspace_root(None, &guard, None)?
    };
    let path = resolve_resource_path_for_write(&workspace_root, &payload.path);
    write_terminology(&path, &payload.payload)
}

#[tauri::command]
pub async fn start_translation(
    app: tauri::AppHandle,
    state: tauri::State<'_, Arc<Mutex<AppState>>>,
    payload: Option<TranslationRunOptions>,
) -> Result<TranslationTask, UserFacingError> {
    let (workspace_root, dry_run) = {
        let guard = state.lock().await;
        let root = resolve_workspace_root(
            Some(&app),
            &guard,
            payload
                .as_ref()
                .and_then(|value| value.workspace_root.clone()),
        )?;
        let dry_run = payload
            .as_ref()
            .and_then(|value| value.dry_run)
            .unwrap_or(false);
        (root, dry_run)
    };

    start_translation_task(app, state.inner().clone(), workspace_root, dry_run).await
}

#[tauri::command]
pub async fn cancel_translation(
    state: tauri::State<'_, Arc<Mutex<AppState>>>,
    task_id: String,
) -> Result<TranslationTask, UserFacingError> {
    let mut guard = state.lock().await;
    let record = guard
        .task
        .as_mut()
        .ok_or_else(|| UserFacingError::new("Task Missing", "No active task found.", None))?;

    if record.task.task_id != task_id {
        return Err(UserFacingError::new(
            "Task Mismatch",
            "The requested task does not match the active task.",
            None,
        ));
    }

    record.cancelled = true;
    record.task.status = crate::backend::models::TaskStatus::Cancelling;
    Ok(record.task.clone())
}

#[tauri::command]
pub async fn get_task_status(
    state: tauri::State<'_, Arc<Mutex<AppState>>>,
    task_id: Option<String>,
) -> Result<Option<TranslationTask>, UserFacingError> {
    let guard = state.lock().await;
    let task = guard.task.as_ref().map(|record| record.task.clone());
    if let (Some(requested), Some(current)) = (task_id, &task) {
        if current.task_id != requested {
            return Ok(None);
        }
    }
    Ok(task)
}
