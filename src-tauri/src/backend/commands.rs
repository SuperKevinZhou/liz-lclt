use std::path::PathBuf;
use std::sync::Arc;

use tokio::sync::Mutex;

use crate::backend::config::{
    default_workspace_paths, load_payload, save_app_config, save_blacklist as persist_blacklist,
    save_models as persist_models, save_translation_configs as persist_translation_configs,
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
    state: &AppState,
    explicit: Option<String>,
) -> Result<PathBuf, UserFacingError> {
    if let Some(path) = explicit {
        return Ok(PathBuf::from(path));
    }

    state.workspace_root.clone().ok_or_else(|| {
        UserFacingError::new(
            "Workspace Required",
            "Select a workspace root before using this action.",
            None,
        )
    })
}

#[tauri::command]
pub async fn set_workspace_root(
    state: tauri::State<'_, Arc<Mutex<AppState>>>,
    payload: WorkspaceSelection,
) -> Result<String, UserFacingError> {
    let mut guard = state.lock().await;
    guard.workspace_root = Some(PathBuf::from(&payload.workspace_root));
    Ok(payload.workspace_root)
}

#[tauri::command]
pub async fn load_app_state(
    state: tauri::State<'_, Arc<Mutex<AppState>>>,
    workspace_root: Option<String>,
) -> Result<AppStatePayload, UserFacingError> {
    let root = {
        let mut guard = state.lock().await;
        let root = resolve_workspace_root(&guard, workspace_root)?;
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
    let workspace_root = resolve_workspace_root(&guard, None)?;
    let relative_path = payload.path.clone();
    let path = workspace_root.join(&relative_path);
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
        resolve_workspace_root(&guard, None)?
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
        resolve_workspace_root(&guard, None)?
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
        resolve_workspace_root(&guard, None)?
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
        resolve_workspace_root(&guard, None)?
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
        resolve_workspace_root(&guard, None)?
    };
    let path = workspace_root.join(payload.path);
    write_text_file(&path, &payload.content)
}

#[tauri::command]
pub async fn save_terminology(
    state: tauri::State<'_, Arc<Mutex<AppState>>>,
    payload: SaveTerminologyPayload,
) -> Result<(), UserFacingError> {
    let workspace_root = {
        let guard = state.lock().await;
        resolve_workspace_root(&guard, None)?
    };
    let path = workspace_root.join(payload.path);
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
            &guard,
            payload.as_ref().and_then(|value| value.workspace_root.clone()),
        )?;
        let dry_run = payload.as_ref().and_then(|value| value.dry_run).unwrap_or(false);
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
