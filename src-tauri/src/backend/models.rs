use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use crate::backend::error::UserFacingError;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    pub translation_settings: TranslationSettings,
    pub file_paths: FilePaths,
    pub config_files: ConfigFiles,
    pub options: AppOptions,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            translation_settings: TranslationSettings {
                origin_language: "jp".into(),
                target_direction: "LCLT_zh".into(),
                max_workers: 256,
                max_chars_per_batch: 2000,
                max_retries: 4,
                timeout: 90,
            },
            file_paths: FilePaths {
                input_direction: String::new(),
                output_direction: String::new(),
            },
            config_files: ConfigFiles {
                models: "models.json".into(),
                translation_configs: "translation_configs.json".into(),
            },
            options: AppOptions {
                keep_backup_files: false,
                confirm_before_translation: true,
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TranslationSettings {
    pub origin_language: String,
    pub target_direction: String,
    pub max_workers: usize,
    pub max_chars_per_batch: usize,
    pub max_retries: usize,
    pub timeout: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FilePaths {
    pub input_direction: String,
    pub output_direction: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigFiles {
    pub models: String,
    pub translation_configs: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppOptions {
    pub keep_backup_files: bool,
    pub confirm_before_translation: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelsConfig {
    pub models: BTreeMap<String, ModelProfile>,
}

impl Default for ModelsConfig {
    fn default() -> Self {
        let mut models = BTreeMap::new();
        models.insert(
            "origin".into(),
            ModelProfile {
                api_key: String::new(),
                base_url: String::new(),
                model: String::new(),
                temperature: 0.3,
                enable_thinking: false,
            },
        );

        Self { models }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelProfile {
    pub api_key: String,
    pub base_url: String,
    pub model: String,
    pub temperature: f64,
    pub enable_thinking: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TranslationConfigs {
    pub translation_strategies: Vec<TranslationStrategy>,
}

impl Default for TranslationConfigs {
    fn default() -> Self {
        Self {
            translation_strategies: vec![],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TranslationStrategy {
    pub name: String,
    pub priority: i64,
    #[serde(default)]
    pub file_patterns: Vec<FilePatternRule>,
    pub model: String,
    pub prompt_file: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub terminology_file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extract_fields: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FilePatternRule {
    pub pattern: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extract_fields: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlacklistConfig {
    #[serde(rename = "BlackList")]
    pub blacklist: Vec<String>,
}

impl Default for BlacklistConfig {
    fn default() -> Self {
        Self { blacklist: vec![] }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TerminologyDictionary {
    pub terminology: BTreeMap<String, String>,
}

impl Default for TerminologyDictionary {
    fn default() -> Self {
        Self {
            terminology: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppStatePayload {
    pub workspace_root: String,
    pub current_config: AppConfig,
    pub models_config: ModelsConfig,
    pub translation_configs: TranslationConfigs,
    pub blacklist_config: BlacklistConfig,
    pub prompt_files: Vec<ResourceFile>,
    pub terminology_files: Vec<ResourceFile>,
    pub auto_detected_game: Option<DetectedGamePaths>,
    pub problems: Vec<UserFacingError>,
    pub current_task: Option<TranslationTask>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceFile {
    pub path: String,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DetectedGamePaths {
    pub steam_library_root: String,
    pub game_root: String,
    pub localize_root: String,
    pub lang_root: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceSelection {
    pub workspace_root: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextResourcePayload {
    pub path: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveTextResourcePayload {
    pub path: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveTerminologyPayload {
    pub path: String,
    pub payload: TerminologyDictionary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TranslationRunOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_root: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dry_run: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TranslationTask {
    pub task_id: String,
    pub status: TaskStatus,
    pub progress: TaskProgressSnapshot,
    pub logs: Vec<TaskLogEntry>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<TaskSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TaskStatus {
    Idle,
    Running,
    Cancelling,
    Cancelled,
    Succeeded,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskProgressSnapshot {
    pub scanned_files: usize,
    pub pending_chars: usize,
    pub completed_batches: usize,
    pub total_batches: usize,
    pub elapsed_ms: u128,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_directory: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskLogEntry {
    pub level: LogLevel,
    pub message: String,
    pub timestamp_ms: u128,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LogLevel {
    Info,
    Warn,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskSummary {
    pub translated_files: usize,
    pub translated_entries: usize,
    pub pending_chars: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_directory: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backup_directory: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub log_file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<UserFacingError>,
}

#[derive(Debug, Clone)]
pub struct WorkspacePaths {
    pub root: PathBuf,
    pub config: PathBuf,
    pub models: PathBuf,
    pub translation_configs: PathBuf,
    pub blacklist: PathBuf,
    pub prompts_dir: PathBuf,
    pub terminology_dir: PathBuf,
}

impl WorkspacePaths {
    pub fn from_root(root: PathBuf, app_config: &AppConfig) -> Self {
        Self {
            root: root.clone(),
            config: root.join("config.json"),
            models: root.join(&app_config.config_files.models),
            translation_configs: root.join(&app_config.config_files.translation_configs),
            blacklist: root.join("BlackList.json"),
            prompts_dir: root.join("prompts"),
            terminology_dir: root.join("terminology"),
        }
    }
}
