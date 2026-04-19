use std::fs;
use std::path::{Path, PathBuf};

use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::backend::error::UserFacingError;
use crate::backend::models::{
    AppConfig, AppStatePayload, AutoDetectedNotice, BlacklistConfig, DetectedGamePaths,
    ModelsConfig, ResourceFile, TranslationConfigs, WorkspacePaths,
};

const ORIGINAL_LCLT_ROOT: &str = "..\\LimbusCompanyLLMTranslator";

fn read_json_file<T: DeserializeOwned>(path: &Path) -> Result<T, UserFacingError> {
    let content =
        fs::read_to_string(path).map_err(|error| UserFacingError::io("Read", path, &error))?;
    serde_json::from_str(&content).map_err(|error| UserFacingError::invalid_json(path, error))
}

fn write_json_file<T: Serialize>(path: &Path, value: &T) -> Result<(), UserFacingError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| UserFacingError::io("Create directory", parent, &error))?;
    }

    let content = serde_json::to_string_pretty(value).map_err(|error| {
        UserFacingError::new(
            "Serialization Error",
            format!("Failed to serialize {}", path.display()),
            Some(error.to_string()),
        )
    })?;

    fs::write(path, content).map_err(|error| UserFacingError::io("Write", path, &error))
}

fn list_files_in_dir(path: &Path, extension: &str) -> Result<Vec<ResourceFile>, UserFacingError> {
    if !path.exists() {
        return Ok(vec![]);
    }

    let mut items = vec![];
    for entry in
        fs::read_dir(path).map_err(|error| UserFacingError::io("Read directory", path, &error))?
    {
        let entry = entry.map_err(|error| {
            UserFacingError::new(
                "Directory Error",
                "Failed to inspect directory entry.",
                Some(error.to_string()),
            )
        })?;
        let entry_path = entry.path();
        if !entry_path.is_file() {
            continue;
        }

        let matches_extension = entry_path
            .extension()
            .and_then(|value| value.to_str())
            .map(|value| value.eq_ignore_ascii_case(extension))
            .unwrap_or(false);
        if !matches_extension {
            continue;
        }

        let relative = entry_path
            .strip_prefix(path.parent().unwrap_or(path))
            .unwrap_or(&entry_path)
            .to_string_lossy()
            .replace('\\', "/");
        let label = entry_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default()
            .to_string();

        items.push(ResourceFile {
            path: relative,
            label,
        });
    }

    items.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(items)
}

fn merge_resource_files(
    primary: Vec<ResourceFile>,
    fallback: Vec<ResourceFile>,
) -> Vec<ResourceFile> {
    let mut merged = primary;
    for item in fallback {
        if !merged.iter().any(|existing| existing.path == item.path) {
            merged.push(item);
        }
    }
    merged.sort_by(|left, right| left.path.cmp(&right.path));
    merged
}

fn original_lclt_root() -> Option<PathBuf> {
    let root = PathBuf::from(ORIGINAL_LCLT_ROOT);
    if root.exists() {
        Some(root)
    } else {
        None
    }
}

pub fn default_workspace_paths(root: PathBuf) -> WorkspacePaths {
    WorkspacePaths::from_root(root, &AppConfig::default())
}

pub fn load_or_default_app_config(path: &Path) -> Result<AppConfig, UserFacingError> {
    if path.exists() {
        read_json_file(path)
    } else {
        Ok(AppConfig::default())
    }
}

pub fn load_models(paths: &WorkspacePaths) -> Result<ModelsConfig, UserFacingError> {
    if paths.models.exists() {
        read_json_file(&paths.models)
    } else {
        Ok(ModelsConfig::default())
    }
}

pub fn load_translation_configs(
    paths: &WorkspacePaths,
) -> Result<TranslationConfigs, UserFacingError> {
    if paths.translation_configs.exists() {
        let loaded: TranslationConfigs = read_json_file(&paths.translation_configs)?;
        if !loaded.translation_strategies.is_empty() {
            return Ok(loaded);
        }
    } else {
        return load_original_translation_configs().or(Ok(TranslationConfigs::default()));
    }

    load_original_translation_configs().or(Ok(TranslationConfigs::default()))
}

pub fn load_blacklist(paths: &WorkspacePaths) -> Result<BlacklistConfig, UserFacingError> {
    if paths.blacklist.exists() {
        read_json_file(&paths.blacklist)
    } else {
        Ok(BlacklistConfig::default())
    }
}

pub fn save_app_config(path: &Path, value: &AppConfig) -> Result<(), UserFacingError> {
    write_json_file(path, value)
}

pub fn save_models(path: &Path, value: &ModelsConfig) -> Result<(), UserFacingError> {
    write_json_file(path, value)
}

pub fn save_translation_configs(
    path: &Path,
    value: &TranslationConfigs,
) -> Result<(), UserFacingError> {
    write_json_file(path, value)
}

pub fn save_blacklist(path: &Path, value: &BlacklistConfig) -> Result<(), UserFacingError> {
    write_json_file(path, value)
}

pub fn load_payload(root: PathBuf) -> AppStatePayload {
    let mut problems = vec![];
    let paths = default_workspace_paths(root.clone());

    let current_config = match load_or_default_app_config(&paths.config) {
        Ok(config) => config,
        Err(error) => {
            problems.push(error);
            AppConfig::default()
        }
    };
    let paths = WorkspacePaths::from_root(root.clone(), &current_config);
    let auto_detected_game = detect_limbus_paths();
    let (current_config, auto_detected_notice) =
        hydrate_config_paths(current_config, auto_detected_game.as_ref());

    let models_config = match load_models(&paths) {
        Ok(config) => config,
        Err(error) => {
            problems.push(error);
            ModelsConfig::default()
        }
    };

    let translation_configs = match load_translation_configs(&paths) {
        Ok(config) => config,
        Err(error) => {
            problems.push(error);
            TranslationConfigs::default()
        }
    };

    let blacklist_config = match load_blacklist(&paths) {
        Ok(config) => config,
        Err(error) => {
            problems.push(error);
            BlacklistConfig::default()
        }
    };

    let fallback_paths = original_lclt_root().map(default_workspace_paths);

    let prompt_files = match list_files_in_dir(&paths.prompts_dir, "txt") {
        Ok(files) => {
            let fallback = fallback_paths
                .as_ref()
                .map(|paths| list_files_in_dir(&paths.prompts_dir, "txt"))
                .transpose();
            match fallback {
                Ok(Some(fallback_files)) => merge_resource_files(files, fallback_files),
                Ok(None) => files,
                Err(error) => {
                    problems.push(error);
                    files
                }
            }
        }
        Err(error) => {
            problems.push(error);
            vec![]
        }
    };

    let terminology_files = match list_files_in_dir(&paths.terminology_dir, "json") {
        Ok(files) => {
            let fallback = fallback_paths
                .as_ref()
                .map(|paths| list_files_in_dir(&paths.terminology_dir, "json"))
                .transpose();
            match fallback {
                Ok(Some(fallback_files)) => merge_resource_files(files, fallback_files),
                Ok(None) => files,
                Err(error) => {
                    problems.push(error);
                    files
                }
            }
        }
        Err(error) => {
            problems.push(error);
            vec![]
        }
    };

    AppStatePayload {
        workspace_root: root.to_string_lossy().to_string(),
        current_config,
        models_config,
        translation_configs,
        blacklist_config,
        prompt_files,
        terminology_files,
        auto_detected_game,
        auto_detected_notice,
        problems,
        current_task: None,
    }
}

fn hydrate_config_paths(
    mut config: AppConfig,
    detected: Option<&DetectedGamePaths>,
) -> (AppConfig, Option<AutoDetectedNotice>) {
    let Some(detected) = detected else {
        return (config, None);
    };

    let mut input_applied = false;
    if config.file_paths.input_direction.trim().is_empty()
        || config
            .file_paths
            .input_direction
            .contains("<Puts your Game Directory Here>")
    {
        config.file_paths.input_direction = detected.localize_root.clone();
        input_applied = true;
    }
    let mut output_applied = false;
    if config.file_paths.output_direction.trim().is_empty()
        || config
            .file_paths
            .output_direction
            .contains("<Puts your Game Directory Here>")
    {
        config.file_paths.output_direction = detected.lang_root.clone();
        output_applied = true;
    }

    let notice = if input_applied || output_applied {
        Some(AutoDetectedNotice {
            game_root: detected.game_root.clone(),
            input_applied,
            output_applied,
        })
    } else {
        None
    };

    (config, notice)
}

fn detect_limbus_paths() -> Option<DetectedGamePaths> {
    let steam_root = detect_steam_root()?;
    let libraries = steam_library_roots(&steam_root);

    for library in libraries {
        let game_root = library
            .join("steamapps")
            .join("common")
            .join("Limbus Company");
        let localize_root = game_root
            .join("LimbusCompany_Data")
            .join("Assets")
            .join("Resources_moved")
            .join("Localize");
        let lang_root = game_root.join("LimbusCompany_Data").join("Lang");

        if localize_root.exists() && lang_root.exists() {
            return Some(DetectedGamePaths {
                steam_library_root: library.to_string_lossy().to_string(),
                game_root: game_root.to_string_lossy().to_string(),
                localize_root: localize_root.to_string_lossy().to_string(),
                lang_root: lang_root.to_string_lossy().to_string(),
            });
        }
    }

    None
}

fn detect_steam_root() -> Option<PathBuf> {
    let candidates = [
        std::env::var_os("PROGRAMFILES(X86)").map(PathBuf::from),
        std::env::var_os("PROGRAMFILES").map(PathBuf::from),
        Some(PathBuf::from("C:\\Program Files (x86)")),
        Some(PathBuf::from("C:\\Program Files")),
    ];

    for base in candidates.into_iter().flatten() {
        let candidate = base.join("Steam");
        if candidate.exists() {
            return Some(candidate);
        }
    }

    None
}

fn steam_library_roots(steam_root: &Path) -> Vec<PathBuf> {
    let mut libraries = vec![steam_root.to_path_buf()];
    let library_vdf = steam_root.join("steamapps").join("libraryfolders.vdf");
    let Ok(content) = fs::read_to_string(&library_vdf) else {
        return libraries;
    };

    for line in content.lines() {
        let trimmed = line.trim();
        if !trimmed.contains("\"path\"") {
            continue;
        }
        let parts = trimmed.split('"').collect::<Vec<_>>();
        let Some(raw_path) = parts
            .iter()
            .rev()
            .find(|part| !part.trim().is_empty() && !part.contains("path"))
        else {
            continue;
        };
        let normalized = raw_path.replace("\\\\", "\\");
        let path = PathBuf::from(normalized);
        if path.exists() && !libraries.contains(&path) {
            libraries.push(path);
        }
    }

    libraries
}

fn load_original_translation_configs() -> Result<TranslationConfigs, UserFacingError> {
    let Some(root) = original_lclt_root() else {
        return Ok(TranslationConfigs::default());
    };
    let path = root.join("translation_configs.json");
    if !path.exists() {
        return Ok(TranslationConfigs::default());
    }
    read_json_file(&path)
}
