use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use futures::stream::{self, StreamExt};
use reqwest::Client;
use serde_json::{json, Value};
use tauri::Emitter;
use tokio::sync::Mutex;

use crate::backend::config::{
    load_blacklist, load_models, load_or_default_app_config, load_translation_configs,
    resolve_existing_resource_path,
};
use crate::backend::error::UserFacingError;
use crate::backend::models::{
    LogLevel, ModelProfile, TaskLogEntry, TaskProgressSnapshot, TaskStatus, TaskSummary,
    TerminologyDictionary, TranslationStrategy, TranslationTask, WorkspacePaths,
};
use crate::backend::resources::{read_terminology, read_text_file};
use crate::backend::state::{AppState, TranslationTaskRecord};
use crate::backend::translator::{
    apply_terminology, execute_batch, plan_batches, TranslationRuntime, TranslationUnit,
};

const TASK_PROGRESS_EVENT: &str = "task_progress";
const TASK_LOG_EVENT: &str = "task_log";
const TASK_FINISHED_EVENT: &str = "task_finished";
const EXTREME_CONCURRENCY_CAP: usize = 32_768;
const VERBOSE_BATCH_LOG_LIMIT: usize = 512;
const PROGRESS_LOG_INTERVAL: usize = 1_000;
const MAX_IN_MEMORY_LOG_ENTRIES: usize = 2_000;

#[derive(Debug, Clone)]
struct FileDelta {
    rel_path: String,
    output_path: PathBuf,
    content: Value,
}

#[derive(Debug, Clone)]
struct ExtractedEntry {
    path: Vec<PathSegment>,
    text: String,
}

#[derive(Debug, Clone)]
struct PendingReplacement {
    unit_index: usize,
    path: Vec<PathSegment>,
    fallback_text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum PathSegment {
    Key(String),
    Index(usize),
}

pub async fn start_translation_task(
    app: tauri::AppHandle,
    state: Arc<Mutex<AppState>>,
    workspace_root: PathBuf,
    dry_run: bool,
) -> Result<TranslationTask, UserFacingError> {
    let config = load_or_default_app_config(&workspace_root.join("config.json"))?;
    let paths = WorkspacePaths::from_root(workspace_root.clone(), &config);
    let models = load_models(&paths)?;
    let translation_configs = load_translation_configs(&paths)?;
    let blacklist = load_blacklist(&paths)?.blacklist;

    validate_references(
        &paths,
        &translation_configs.translation_strategies,
        &models.models,
    )?;

    let task_id = format!("task-{}", now_ms());
    let output_root = workspace_root
        .join(&config.file_paths.output_direction)
        .join(&config.translation_settings.target_direction);
    let backup_dir = workspace_root.join("backup");
    let log_file = workspace_root
        .join("backup")
        .join(format!("task_log_{}.log", now_ms()));

    let mut task = TranslationTask {
        task_id: task_id.clone(),
        status: TaskStatus::Running,
        progress: TaskProgressSnapshot {
            scanned_files: 0,
            pending_chars: 0,
            completed_batches: 0,
            total_batches: 0,
            elapsed_ms: 0,
            output_directory: Some(output_root.to_string_lossy().to_string()),
        },
        logs: vec![],
        summary: None,
    };

    {
        let mut guard = state.lock().await;
        guard.task = Some(TranslationTaskRecord {
            task: task.clone(),
            started_at: Instant::now(),
            cancelled: false,
        });
    }

    push_log(
        &app,
        &state,
        LogLevel::Info,
        "Scanning source and target language directories.",
        Some(&log_file),
    )
    .await?;
    let input_root = workspace_root
        .join(&config.file_paths.input_direction)
        .join(&config.translation_settings.origin_language);

    let origin_files = scan_json_files(&input_root)?;
    let existing_files = scan_json_files(&output_root)?;
    let mut deltas = find_deltas(&origin_files, &existing_files, &input_root, &output_root)?;

    push_log(
        &app,
        &state,
        LogLevel::Info,
        format!("Found {} files with new entries.", deltas.len()),
        Some(&log_file),
    )
    .await?;

    let prompt_cache = load_prompt_cache(&paths, &translation_configs.translation_strategies)?;
    let terminology_cache =
        load_terminology_cache(&paths, &translation_configs.translation_strategies)?;

    let mut all_units: Vec<TranslationUnit> = vec![];
    let mut replacements_by_file: Vec<Vec<PendingReplacement>> = vec![vec![]; deltas.len()];
    let mut translated_entries = 0usize;

    for (file_index, delta) in deltas.iter().enumerate() {
        let strategies =
            matching_strategies(&translation_configs.translation_strategies, &delta.rel_path);
        let Some(items) = delta
            .content
            .get("dataList")
            .and_then(|value| value.as_array())
        else {
            continue;
        };

        let mut processed_paths = BTreeSet::new();
        for (item_index, item) in items.iter().enumerate() {
            for strategy in &strategies {
                let extract_fields = strategy.extract_fields.clone().or_else(|| {
                    strategy
                        .file_patterns
                        .iter()
                        .find_map(|rule| rule.extract_fields.clone())
                });
                let extracted = extract_text_recursive(
                    item,
                    &blacklist,
                    vec![PathSegment::Index(item_index)],
                    extract_fields.as_deref(),
                );

                for entry in extracted {
                    if !processed_paths.insert(entry.path.clone()) {
                        continue;
                    }

                    let model_profile = models.models.get(&strategy.model).ok_or_else(|| {
                        UserFacingError::new(
                            "Invalid Strategy Reference",
                            format!(
                                "Strategy '{}' references missing model '{}'.",
                                strategy.name, strategy.model
                            ),
                            None,
                        )
                    })?;

                    let terminology = strategy
                        .terminology_file
                        .as_deref()
                        .and_then(|path| terminology_cache.get(path))
                        .cloned()
                        .unwrap_or_else(|| TerminologyDictionary {
                            terminology: BTreeMap::new(),
                        });
                    let prepared_text = apply_terminology(&entry.text, &terminology.terminology);
                    let prompt_text = prompt_cache
                        .get(&strategy.prompt_file)
                        .cloned()
                        .unwrap_or_default();

                    let unit_index = all_units.len();
                    all_units.push(TranslationUnit {
                        unit_index,
                        prepared_text,
                        runtime: TranslationRuntime {
                            api_key: model_profile.api_key.clone(),
                            base_url: model_profile.base_url.clone(),
                            model: model_profile.model.clone(),
                            temperature: model_profile.temperature,
                            enable_thinking: model_profile.enable_thinking,
                            prompt_file: strategy.prompt_file.clone(),
                            prompt_text,
                        },
                    });
                    replacements_by_file[file_index].push(PendingReplacement {
                        unit_index,
                        path: entry.path,
                        fallback_text: entry.text,
                    });
                    translated_entries += 1;
                }
            }
        }
    }

    let batches = plan_batches(&all_units, config.translation_settings.max_chars_per_batch);
    update_progress(&app, &state, |progress| {
        progress.scanned_files = origin_files.len();
        progress.pending_chars = all_units
            .iter()
            .map(|unit| unit.prepared_text.as_bytes().len())
            .sum::<usize>();
        progress.total_batches = batches.len();
    })
    .await?;

    if deltas.is_empty() || all_units.is_empty() {
        finish_task(
            &app,
            &state,
            TaskStatus::Succeeded,
            TaskSummary {
                translated_files: 0,
                translated_entries: 0,
                pending_chars: 0,
                output_directory: Some(output_root.to_string_lossy().to_string()),
                backup_directory: Some(backup_dir.to_string_lossy().to_string()),
                log_file: Some(log_file.to_string_lossy().to_string()),
                error: None,
            },
        )
        .await?;
        let guard = state.lock().await;
        return Ok(guard.task.as_ref().unwrap().task.clone());
    }

    let requested_workers = config.translation_settings.max_workers.max(1);
    let concurrency = requested_workers
        .min(EXTREME_CONCURRENCY_CAP)
        .min(batches.len().max(1));
    let verbose_batch_logs = batches.len() <= VERBOSE_BATCH_LOG_LIMIT;

    let client = Client::builder()
        .pool_max_idle_per_host(concurrency.max(32))
        .tcp_keepalive(Some(std::time::Duration::from_secs(30)))
        .connect_timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|error| {
            UserFacingError::new(
                "HTTP Client Error",
                "Failed to create HTTP client.",
                Some(error.to_string()),
            )
        })?;

    push_log(
        &app,
        &state,
        LogLevel::Info,
        format!(
            "Prepared {} translation units across {} batch(es). Requested workers: {}; active concurrent batches: {}; log mode: {}.",
            all_units.len(),
            batches.len(),
            config.translation_settings.max_workers,
            concurrency,
            if verbose_batch_logs { "verbose" } else { "aggregated" }
        ),
        Some(&log_file),
    )
    .await?;

    let app_for_batches = app.clone();
    let state_for_batches = state.clone();
    let log_path_for_batches = log_file.clone();
    if requested_workers != concurrency {
        push_log(
            &app,
            &state,
            LogLevel::Warn,
            format!(
                "Requested max_workers={} exceeds the Rust extreme cap or batch count. Running with {} concurrent batches.",
                requested_workers, concurrency
            ),
            Some(&log_file),
        )
        .await?;
    }

    let batch_stream = stream::iter(batches.clone())
        .map(|batch| {
            let client = client.clone();
            let app = app_for_batches.clone();
            let state = state_for_batches.clone();
            let log_path = log_path_for_batches.clone();
            async move {
                if is_cancelled(&state).await {
                    return Ok::<BatchOutcome, UserFacingError>(BatchOutcome {
                        translations: batch
                            .unit_indices
                            .iter()
                            .enumerate()
                            .map(|(offset, unit_index)| (*unit_index, batch.texts[offset].clone()))
                            .collect(),
                        used_fallback: true,
                    });
                }

                if verbose_batch_logs {
                    push_log(
                        &app,
                        &state,
                        LogLevel::Info,
                        format!(
                            "Dispatching batch {} with {} item(s) on model '{}'.",
                            batch.batch_index + 1,
                            batch.unit_indices.len(),
                            batch.runtime.model
                        ),
                        Some(&log_path),
                    )
                    .await?;
                }

                let result = execute_batch(
                    &client,
                    &batch,
                    config.translation_settings.timeout,
                    config.translation_settings.max_retries,
                )
                .await;
                let outcome = match result {
                    Ok(result) => {
                        if verbose_batch_logs {
                            push_log(
                                &app,
                                &state,
                                LogLevel::Info,
                                format!(
                                    "Batch {} completed in {} attempt(s).",
                                    batch.batch_index + 1,
                                    result.attempts
                                ),
                                Some(&log_path),
                            )
                            .await?;
                        }
                        BatchOutcome {
                            translations: result.translations,
                            used_fallback: false,
                        }
                    }
                    Err(error) => {
                        push_log(
                            &app,
                            &state,
                            LogLevel::Warn,
                            format!(
                                "Batch {} failed after retries. Falling back to source text. {}",
                                batch.batch_index + 1,
                                error.message
                            ),
                            Some(&log_path),
                        )
                        .await?;
                        BatchOutcome {
                            translations: batch
                                .unit_indices
                                .iter()
                                .enumerate()
                                .map(|(offset, unit_index)| (*unit_index, batch.texts[offset].clone()))
                                .collect(),
                            used_fallback: true,
                        }
                    }
                };

                update_progress(&app, &state, |progress| {
                    progress.completed_batches += 1;
                })
                .await?;

                if !verbose_batch_logs
                    && (batch.batch_index + 1) % PROGRESS_LOG_INTERVAL == 0
                {
                    push_log(
                        &app,
                        &state,
                        LogLevel::Info,
                        format!(
                            "Processed at least {} batch dispatch slots under aggregated extreme logging.",
                            batch.batch_index + 1
                        ),
                        Some(&log_path),
                    )
                    .await?;
                }

                Ok(outcome)
            }
        })
        .buffer_unordered(concurrency);

    let mut translated_map: HashMap<usize, String> = HashMap::new();
    let mut fallback_batches = 0usize;
    tokio::pin!(batch_stream);
    while let Some(result) = batch_stream.next().await {
        let outcome = result?;
        if outcome.used_fallback {
            fallback_batches += 1;
        }
        translated_map.extend(outcome.translations);
    }

    update_progress(&app, &state, |progress| {
        progress.pending_chars = 0;
    })
    .await?;

    let backup_translation_result_path =
        backup_dir.join(format!("translation_result_{}.json", now_ms()));
    if !dry_run {
        fs::create_dir_all(&backup_dir)
            .map_err(|error| UserFacingError::io("Create directory", &backup_dir, &error))?;
    }

    let mut translated_files = 0usize;
    let mut backup_payload = vec![];
    for (file_index, delta) in deltas.iter_mut().enumerate() {
        let Some(data_list) = delta
            .content
            .get_mut("dataList")
            .and_then(|value| value.as_array_mut())
        else {
            continue;
        };

        for replacement in &replacements_by_file[file_index] {
            let translated_text = translated_map
                .get(&replacement.unit_index)
                .cloned()
                .unwrap_or_else(|| replacement.fallback_text.clone());
            set_text_recursive(data_list, &replacement.path, translated_text)?;
        }

        if !dry_run {
            write_output_file(&delta.output_path, &delta.content)?;
            copy_font_if_missing(&workspace_root, &output_root)?;
            backup_payload.push(json!({
                "rel_path": delta.rel_path,
                "output_path": delta.output_path,
                "content": delta.content,
            }));
        }

        translated_files += 1;
    }

    if !dry_run && config.options.keep_backup_files {
        let backup_content = serde_json::to_string_pretty(&backup_payload).map_err(|error| {
            UserFacingError::new(
                "Serialization Error",
                "Failed to serialize translation backup output.",
                Some(error.to_string()),
            )
        })?;
        fs::write(&backup_translation_result_path, backup_content).map_err(|error| {
            UserFacingError::io("Write", &backup_translation_result_path, &error)
        })?;
    }

    if fallback_batches > 0 {
        push_log(
            &app,
            &state,
            LogLevel::Warn,
            format!(
                "{} batch(es) used source-text fallback because the provider failed or was cancelled.",
                fallback_batches
            ),
            Some(&log_file),
        )
        .await?;
    }

    finish_task(
        &app,
        &state,
        TaskStatus::Succeeded,
        TaskSummary {
            translated_files,
            translated_entries,
            pending_chars: 0,
            output_directory: Some(output_root.to_string_lossy().to_string()),
            backup_directory: Some(backup_dir.to_string_lossy().to_string()),
            log_file: Some(log_file.to_string_lossy().to_string()),
            error: None,
        },
    )
    .await?;

    let guard = state.lock().await;
    task = guard.task.as_ref().unwrap().task.clone();
    Ok(task)
}

#[derive(Debug)]
struct BatchOutcome {
    translations: HashMap<usize, String>,
    used_fallback: bool,
}

fn validate_references(
    paths: &WorkspacePaths,
    strategies: &[TranslationStrategy],
    models: &std::collections::BTreeMap<String, ModelProfile>,
) -> Result<(), UserFacingError> {
    for strategy in strategies {
        if !models.contains_key(&strategy.model) {
            return Err(UserFacingError::new(
                "Invalid Strategy Reference",
                format!(
                    "Strategy '{}' references missing model '{}'.",
                    strategy.name, strategy.model
                ),
                None,
            ));
        }
        if resolve_existing_resource_path(&paths.root, &strategy.prompt_file).is_none() {
            return Err(UserFacingError::new(
                "Missing Prompt",
                format!(
                    "Strategy '{}' references missing prompt '{}'.",
                    strategy.name, strategy.prompt_file
                ),
                None,
            ));
        }
        if let Some(terminology) = &strategy.terminology_file {
            if resolve_existing_resource_path(&paths.root, terminology).is_none() {
                return Err(UserFacingError::new(
                    "Missing Terminology",
                    format!(
                        "Strategy '{}' references missing terminology '{}'.",
                        strategy.name, terminology
                    ),
                    None,
                ));
            }
        }
    }

    Ok(())
}

fn load_prompt_cache(
    paths: &WorkspacePaths,
    strategies: &[TranslationStrategy],
) -> Result<HashMap<String, String>, UserFacingError> {
    let mut cache = HashMap::new();
    for strategy in strategies {
        if cache.contains_key(&strategy.prompt_file) {
            continue;
        }
        let path = resolve_existing_resource_path(&paths.root, &strategy.prompt_file).ok_or_else(
            || {
                UserFacingError::new(
                    "Missing Prompt",
                    format!(
                        "Strategy references missing prompt '{}'.",
                        strategy.prompt_file
                    ),
                    None,
                )
            },
        )?;
        let content = read_text_file(&path)?;
        cache.insert(strategy.prompt_file.clone(), content);
    }
    Ok(cache)
}

fn load_terminology_cache(
    paths: &WorkspacePaths,
    strategies: &[TranslationStrategy],
) -> Result<HashMap<String, TerminologyDictionary>, UserFacingError> {
    let mut cache = HashMap::new();
    for strategy in strategies {
        let Some(path) = &strategy.terminology_file else {
            continue;
        };
        if cache.contains_key(path) {
            continue;
        }
        let resolved_path = resolve_existing_resource_path(&paths.root, path).ok_or_else(|| {
            UserFacingError::new(
                "Missing Terminology",
                format!("Strategy references missing terminology '{}'.", path),
                None,
            )
        })?;
        let terminology = read_terminology(&resolved_path)?;
        cache.insert(path.clone(), terminology);
    }
    Ok(cache)
}

fn scan_json_files(root: &Path) -> Result<HashMap<String, Value>, UserFacingError> {
    let mut files = HashMap::new();
    if !root.exists() {
        return Ok(files);
    }

    let mut stack = vec![root.to_path_buf()];
    while let Some(current) = stack.pop() {
        for entry in fs::read_dir(&current)
            .map_err(|error| UserFacingError::io("Read directory", &current, &error))?
        {
            let entry = entry.map_err(|error| {
                UserFacingError::new(
                    "Directory Error",
                    "Failed to read directory entry.",
                    Some(error.to_string()),
                )
            })?;
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            let is_json = path
                .extension()
                .and_then(|value| value.to_str())
                .map(|value| value.eq_ignore_ascii_case("json"))
                .unwrap_or(false);
            if !is_json {
                continue;
            }

            let raw = fs::read_to_string(&path)
                .map_err(|error| UserFacingError::io("Read", &path, &error))?;
            let value: Value = serde_json::from_str(&raw)
                .map_err(|error| UserFacingError::invalid_json(&path, error))?;

            let relative = path
                .strip_prefix(root)
                .unwrap_or(&path)
                .to_string_lossy()
                .replace('\\', "/");
            files.insert(relative, value);
        }
    }

    Ok(files)
}

fn find_deltas(
    origin_files: &HashMap<String, Value>,
    existing_files: &HashMap<String, Value>,
    _input_root: &Path,
    output_root: &Path,
) -> Result<Vec<FileDelta>, UserFacingError> {
    let mut deltas = vec![];

    let existing_normalized = existing_files
        .iter()
        .map(|(rel_path, value)| (normalize_rel_path(rel_path), value))
        .collect::<HashMap<_, _>>();

    for (rel_path, origin_value) in origin_files {
        let normalized_rel_path = normalize_rel_path(rel_path);
        let Some(origin_data) = origin_value
            .get("dataList")
            .and_then(|value| value.as_array())
        else {
            continue;
        };

        let existing_ids = existing_normalized
            .get(&normalized_rel_path)
            .and_then(|value| value.get("dataList"))
            .and_then(|value| value.as_array())
            .map(|items| {
                items
                    .iter()
                    .filter_map(|item| item.get("id").and_then(|value| value.as_i64()))
                    .collect::<BTreeSet<_>>()
            })
            .unwrap_or_default();

        let mut new_items = vec![];
        for item in origin_data {
            let Some(id) = item.get("id").and_then(|value| value.as_i64()) else {
                new_items.push(item.clone());
                continue;
            };
            if !existing_ids.contains(&id) {
                new_items.push(item.clone());
            }
        }

        if new_items.is_empty() {
            continue;
        }

        let mut content = origin_value.clone();
        if let Some(map) = content.as_object_mut() {
            map.insert("dataList".into(), Value::Array(new_items));
        }

        deltas.push(FileDelta {
            rel_path: normalized_rel_path.clone(),
            output_path: output_root.join(&normalized_rel_path),
            content,
        });
    }

    Ok(deltas)
}

fn normalize_rel_path(path: &str) -> String {
    let path = path.replace('\\', "/");
    let parts = path
        .split('/')
        .map(normalize_filename_prefix)
        .collect::<Vec<_>>();
    if parts.is_empty() {
        return path;
    }
    parts.join("/")
}

fn normalize_filename_prefix(segment: &str) -> String {
    let mut current = segment.to_string();
    loop {
        let next = if let Some(stripped) = current.strip_prefix("KR_") {
            stripped.to_string()
        } else if let Some(stripped) = current.strip_prefix("JP_") {
            stripped.to_string()
        } else if let Some(stripped) = current.strip_prefix("EN_") {
            stripped.to_string()
        } else {
            break;
        };
        current = next;
    }
    current
}

fn matching_strategies(
    strategies: &[TranslationStrategy],
    rel_path: &str,
) -> Vec<TranslationStrategy> {
    let mut sorted = strategies.to_vec();
    sorted.sort_by_key(|strategy| strategy.priority);
    let file_name = Path::new(rel_path)
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or(rel_path);

    let mut matched = vec![];
    for strategy in sorted {
        for rule in &strategy.file_patterns {
            if wildcard_matches(&rule.pattern, rel_path)
                || wildcard_matches(&rule.pattern, file_name)
            {
                let mut strategy = strategy.clone();
                if strategy.extract_fields.is_none() {
                    strategy.extract_fields = rule.extract_fields.clone();
                }
                matched.push(strategy);
                break;
            }
        }
    }

    if matched.is_empty() {
        if let Some(default) = strategies
            .iter()
            .find(|strategy| strategy.name == "default")
        {
            matched.push(default.clone());
        }
    }

    matched
}

fn wildcard_matches(pattern: &str, text: &str) -> bool {
    fn inner(pattern: &[u8], text: &[u8]) -> bool {
        if pattern.is_empty() {
            return text.is_empty();
        }
        match pattern[0] {
            b'*' => inner(&pattern[1..], text) || (!text.is_empty() && inner(pattern, &text[1..])),
            b'?' => !text.is_empty() && inner(&pattern[1..], &text[1..]),
            byte => !text.is_empty() && byte == text[0] && inner(&pattern[1..], &text[1..]),
        }
    }

    inner(pattern.as_bytes(), text.as_bytes())
}

fn extract_text_recursive(
    data: &Value,
    blacklist: &[String],
    current_path: Vec<PathSegment>,
    extract_fields: Option<&[String]>,
) -> Vec<ExtractedEntry> {
    let mut items = vec![];
    match data {
        Value::Object(map) => {
            for (key, value) in map {
                if blacklist.iter().any(|blocked| blocked == key) {
                    continue;
                }
                let mut path = current_path.clone();
                path.push(PathSegment::Key(key.clone()));
                if extract_fields
                    .map(|fields| fields.iter().any(|field| field == key))
                    .unwrap_or(false)
                    && value.is_string()
                {
                    items.push(ExtractedEntry {
                        path,
                        text: value.as_str().unwrap_or_default().to_string(),
                    });
                } else {
                    items.extend(extract_text_recursive(
                        value,
                        blacklist,
                        path,
                        extract_fields,
                    ));
                }
            }
        }
        Value::Array(list) => {
            for (index, value) in list.iter().enumerate() {
                let mut path = current_path.clone();
                path.push(PathSegment::Index(index));
                items.extend(extract_text_recursive(
                    value,
                    blacklist,
                    path,
                    extract_fields,
                ));
            }
        }
        Value::String(text) if extract_fields.is_none() => {
            items.push(ExtractedEntry {
                path: current_path,
                text: text.clone(),
            });
        }
        _ => {}
    }

    items
}

fn set_text_recursive(
    data_list: &mut [Value],
    path: &[PathSegment],
    replacement: String,
) -> Result<(), UserFacingError> {
    let Some((first, rest)) = path.split_first() else {
        return Ok(());
    };

    let target_index = match first {
        PathSegment::Index(index) => *index,
        PathSegment::Key(_) => {
            return Err(UserFacingError::new(
                "Path Error",
                "Invalid translation path.",
                None,
            ))
        }
    };

    let Some(target) = data_list.get_mut(target_index) else {
        return Err(UserFacingError::new(
            "Path Error",
            "Could not locate target entry for translated text.",
            None,
        ));
    };

    set_value_recursive(target, rest, replacement)
}

fn set_value_recursive(
    current: &mut Value,
    path: &[PathSegment],
    replacement: String,
) -> Result<(), UserFacingError> {
    if path.is_empty() {
        *current = Value::String(replacement);
        return Ok(());
    }

    match (&path[0], current) {
        (PathSegment::Key(key), Value::Object(map)) => {
            let Some(next) = map.get_mut(key) else {
                return Err(UserFacingError::new(
                    "Path Error",
                    format!("Missing field '{}'.", key),
                    None,
                ));
            };
            set_value_recursive(next, &path[1..], replacement)
        }
        (PathSegment::Index(index), Value::Array(list)) => {
            let Some(next) = list.get_mut(*index) else {
                return Err(UserFacingError::new(
                    "Path Error",
                    format!("Missing list index {}.", index),
                    None,
                ));
            };
            set_value_recursive(next, &path[1..], replacement)
        }
        _ => Err(UserFacingError::new(
            "Path Error",
            "Encountered non-container value while writing translation.",
            None,
        )),
    }
}

fn write_output_file(path: &Path, content: &Value) -> Result<(), UserFacingError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| UserFacingError::io("Create directory", parent, &error))?;
    }
    let content = serde_json::to_string_pretty(content).map_err(|error| {
        UserFacingError::new(
            "Serialization Error",
            format!("Failed to serialize {}", path.display()),
            Some(error.to_string()),
        )
    })?;
    fs::write(path, content).map_err(|error| UserFacingError::io("Write", path, &error))
}

fn copy_font_if_missing(root: &Path, output_root: &Path) -> Result<(), UserFacingError> {
    let source_font_dir = root.join("Font");
    let target_font_dir = output_root.join("Font");
    if target_font_dir.exists() || !source_font_dir.exists() {
        return Ok(());
    }

    copy_dir_all(&source_font_dir, &target_font_dir)
}

fn copy_dir_all(source: &Path, destination: &Path) -> Result<(), UserFacingError> {
    fs::create_dir_all(destination)
        .map_err(|error| UserFacingError::io("Create directory", destination, &error))?;

    for entry in fs::read_dir(source)
        .map_err(|error| UserFacingError::io("Read directory", source, &error))?
    {
        let entry = entry.map_err(|error| {
            UserFacingError::new(
                "Directory Error",
                "Failed to inspect font asset.",
                Some(error.to_string()),
            )
        })?;
        let entry_path = entry.path();
        let target = destination.join(entry.file_name());
        if entry_path.is_dir() {
            copy_dir_all(&entry_path, &target)?;
        } else {
            fs::copy(&entry_path, &target)
                .map_err(|error| UserFacingError::io("Copy", &entry_path, &error))?;
        }
    }

    Ok(())
}

async fn update_progress<F>(
    app: &tauri::AppHandle,
    state: &Arc<Mutex<AppState>>,
    update: F,
) -> Result<(), UserFacingError>
where
    F: FnOnce(&mut TaskProgressSnapshot),
{
    let payload = {
        let mut guard = state.lock().await;
        let record = guard.task.as_mut().ok_or_else(|| {
            UserFacingError::new("Task Missing", "No active task was found.", None)
        })?;
        update(&mut record.task.progress);
        record.task.progress.elapsed_ms = record.started_at.elapsed().as_millis();
        record.task.clone()
    };
    app.emit(TASK_PROGRESS_EVENT, &payload).map_err(|error| {
        UserFacingError::new(
            "Event Error",
            "Failed to emit task progress.",
            Some(error.to_string()),
        )
    })
}

async fn push_log(
    app: &tauri::AppHandle,
    state: &Arc<Mutex<AppState>>,
    level: LogLevel,
    message: impl Into<String>,
    log_path: Option<&Path>,
) -> Result<(), UserFacingError> {
    let message = message.into();
    let payload = {
        let mut guard = state.lock().await;
        let record = guard.task.as_mut().ok_or_else(|| {
            UserFacingError::new("Task Missing", "No active task was found.", None)
        })?;
        let entry = TaskLogEntry {
            level: level.clone(),
            message: message.clone(),
            timestamp_ms: record.started_at.elapsed().as_millis(),
        };
        record.task.logs.push(entry.clone());
        if record.task.logs.len() > MAX_IN_MEMORY_LOG_ENTRIES {
            let overflow = record.task.logs.len() - MAX_IN_MEMORY_LOG_ENTRIES;
            record.task.logs.drain(0..overflow);
        }
        entry
    };

    if let Some(log_path) = log_path {
        append_log_file(log_path, &payload)?;
    }

    app.emit(TASK_LOG_EVENT, &payload).map_err(|error| {
        UserFacingError::new(
            "Event Error",
            "Failed to emit task log.",
            Some(error.to_string()),
        )
    })
}

fn append_log_file(path: &Path, entry: &TaskLogEntry) -> Result<(), UserFacingError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| UserFacingError::io("Create directory", parent, &error))?;
    }
    let line = format!(
        "[{:>8}ms][{:?}] {}\n",
        entry.timestamp_ms, entry.level, entry.message
    );
    use std::io::Write;
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|error| UserFacingError::io("Open", path, &error))?;
    file.write_all(line.as_bytes())
        .map_err(|error| UserFacingError::io("Write", path, &error))
}

async fn finish_task(
    app: &tauri::AppHandle,
    state: &Arc<Mutex<AppState>>,
    status: TaskStatus,
    summary: TaskSummary,
) -> Result<(), UserFacingError> {
    let payload = {
        let mut guard = state.lock().await;
        let record = guard.task.as_mut().ok_or_else(|| {
            UserFacingError::new("Task Missing", "No active task was found.", None)
        })?;
        record.task.status = status;
        record.task.summary = Some(summary);
        record.task.progress.elapsed_ms = record.started_at.elapsed().as_millis();
        record.task.clone()
    };
    app.emit(TASK_FINISHED_EVENT, &payload).map_err(|error| {
        UserFacingError::new(
            "Event Error",
            "Failed to emit task completion.",
            Some(error.to_string()),
        )
    })
}

async fn is_cancelled(state: &Arc<Mutex<AppState>>) -> bool {
    let guard = state.lock().await;
    guard
        .task
        .as_ref()
        .map(|record| record.cancelled)
        .unwrap_or(false)
}

fn now_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}
