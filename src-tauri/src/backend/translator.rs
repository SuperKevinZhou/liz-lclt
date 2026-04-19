use std::collections::{BTreeMap, HashMap};
use std::hash::{Hash, Hasher};
use std::time::Duration;

use regex::Regex;
use reqwest::Client;
use serde_json::{json, Value};

use crate::backend::error::UserFacingError;

#[derive(Debug, Clone)]
pub struct TranslationRuntime {
    pub api_key: String,
    pub base_url: String,
    pub model: String,
    pub temperature: f64,
    pub enable_thinking: bool,
    pub prompt_file: String,
    pub prompt_text: String,
}

#[derive(Debug, Clone)]
pub struct TranslationUnit {
    pub unit_index: usize,
    pub prepared_text: String,
    pub runtime: TranslationRuntime,
}

#[derive(Debug, Clone)]
pub struct TranslationBatch {
    pub batch_index: usize,
    pub unit_indices: Vec<usize>,
    pub texts: Vec<String>,
    pub runtime: TranslationRuntime,
}

#[derive(Debug, Clone)]
pub struct BatchExecutionResult {
    pub translations: HashMap<usize, String>,
    pub attempts: usize,
}

#[derive(Debug, Clone, Eq)]
struct TranslationGroupKey {
    api_key: String,
    base_url: String,
    model: String,
    temperature_bits: u64,
    enable_thinking: bool,
    prompt_file: String,
}

impl PartialEq for TranslationGroupKey {
    fn eq(&self, other: &Self) -> bool {
        self.api_key == other.api_key
            && self.base_url == other.base_url
            && self.model == other.model
            && self.temperature_bits == other.temperature_bits
            && self.enable_thinking == other.enable_thinking
            && self.prompt_file == other.prompt_file
    }
}

impl Hash for TranslationGroupKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.api_key.hash(state);
        self.base_url.hash(state);
        self.model.hash(state);
        self.temperature_bits.hash(state);
        self.enable_thinking.hash(state);
        self.prompt_file.hash(state);
    }
}

pub fn apply_terminology(text: &str, terminology: &BTreeMap<String, String>) -> String {
    if text.is_empty() || terminology.is_empty() {
        return text.to_string();
    }

    let mut entries: Vec<_> = terminology.iter().collect();
    entries.sort_by_key(|(source, _)| std::cmp::Reverse(source.len()));

    let pattern = entries
        .iter()
        .map(|(source, _)| regex::escape(source))
        .collect::<Vec<_>>()
        .join("|");
    if pattern.is_empty() {
        return text.to_string();
    }

    let regex = match Regex::new(&pattern) {
        Ok(regex) => regex,
        Err(_) => return text.to_string(),
    };

    regex
        .replace_all(text, |captures: &regex::Captures| {
            let value = captures.get(0).map(|m| m.as_str()).unwrap_or_default();
            terminology
                .get(value)
                .cloned()
                .unwrap_or_else(|| value.to_string())
        })
        .to_string()
}

pub fn plan_batches(units: &[TranslationUnit], max_chars_per_batch: usize) -> Vec<TranslationBatch> {
    let mut groups: HashMap<TranslationGroupKey, Vec<&TranslationUnit>> = HashMap::new();
    for unit in units {
        let key = TranslationGroupKey {
            api_key: unit.runtime.api_key.clone(),
            base_url: unit.runtime.base_url.clone(),
            model: unit.runtime.model.clone(),
            temperature_bits: unit.runtime.temperature.to_bits(),
            enable_thinking: unit.runtime.enable_thinking,
            prompt_file: unit.runtime.prompt_file.clone(),
        };
        groups.entry(key).or_default().push(unit);
    }

    let mut batches = vec![];
    let mut batch_index = 0usize;

    for group_units in groups.into_values() {
        let mut current_indices = vec![];
        let mut current_texts = vec![];
        let mut current_chars = 0usize;
        let runtime = group_units[0].runtime.clone();

        for unit in group_units {
            let text_chars = unit.prepared_text.as_bytes().len();
            if !current_indices.is_empty() && current_chars + text_chars > max_chars_per_batch {
                batches.push(TranslationBatch {
                    batch_index,
                    unit_indices: current_indices,
                    texts: current_texts,
                    runtime: runtime.clone(),
                });
                batch_index += 1;
                current_indices = vec![];
                current_texts = vec![];
                current_chars = 0;
            }

            current_indices.push(unit.unit_index);
            current_texts.push(unit.prepared_text.clone());
            current_chars += text_chars;
        }

        if !current_indices.is_empty() {
            batches.push(TranslationBatch {
                batch_index,
                unit_indices: current_indices,
                texts: current_texts,
                runtime,
            });
            batch_index += 1;
        }
    }

    batches.sort_by_key(|batch| batch.batch_index);
    batches
}

pub async fn execute_batch(
    client: &Client,
    batch: &TranslationBatch,
    timeout_seconds: u64,
    max_retries: usize,
) -> Result<BatchExecutionResult, UserFacingError> {
    if batch.runtime.api_key.trim().is_empty() {
        return Err(UserFacingError::new(
            "Model Config Error",
            format!("Model '{}' is missing an API key.", batch.runtime.model),
            None,
        ));
    }
    if batch.runtime.base_url.trim().is_empty() {
        return Err(UserFacingError::new(
            "Model Config Error",
            format!("Model '{}' is missing a base URL.", batch.runtime.model),
            None,
        ));
    }
    if batch.runtime.model.trim().is_empty() {
        return Err(UserFacingError::new(
            "Model Config Error",
            "The selected model slot does not declare a model id.",
            None,
        ));
    }

    let user_content = batch
        .texts
        .iter()
        .enumerate()
        .map(|(index, text)| format!("{}. {}\n---\n", index + 1, text))
        .collect::<String>();

    let mut payload = json!({
        "model": batch.runtime.model,
        "messages": [
            { "role": "system", "content": batch.runtime.prompt_text },
            { "role": "user", "content": user_content }
        ],
        "temperature": batch.runtime.temperature,
        "max_tokens": 8192
    });

    if batch.runtime.enable_thinking {
        payload["thinking"] = Value::Bool(true);
    }

    let mut last_error: Option<UserFacingError> = None;
    for attempt in 0..=max_retries {
        let response = client
            .post(&batch.runtime.base_url)
            .bearer_auth(&batch.runtime.api_key)
            .json(&payload)
            .timeout(Duration::from_secs(timeout_seconds))
            .send()
            .await;

        match response {
            Ok(response) => {
                let response = response.error_for_status().map_err(|error| {
                    UserFacingError::new(
                        "Translation Request Failed",
                        format!(
                            "The provider rejected batch {} for model '{}'.",
                            batch.batch_index + 1,
                            batch.runtime.model
                        ),
                        Some(error.to_string()),
                    )
                })?;

                let payload: Value = response.json().await.map_err(|error| {
                    UserFacingError::new(
                        "Translation Response Error",
                        "The provider returned an unreadable JSON payload.",
                        Some(error.to_string()),
                    )
                })?;

                let content = payload
                    .get("choices")
                    .and_then(|value| value.as_array())
                    .and_then(|items| items.first())
                    .and_then(|value| value.get("message"))
                    .and_then(|value| value.get("content"))
                    .and_then(|value| value.as_str())
                    .ok_or_else(|| {
                        UserFacingError::new(
                            "Translation Response Error",
                            "The provider response did not include choices[0].message.content.",
                            Some(payload.to_string()),
                        )
                    })?;

                let parsed = parse_translations(content);
                let translations = batch
                    .unit_indices
                    .iter()
                    .enumerate()
                    .map(|(batch_offset, unit_index)| {
                        let translated = parsed
                            .get(&batch_offset)
                            .cloned()
                            .unwrap_or_else(|| batch.texts[batch_offset].clone());
                        (*unit_index, translated)
                    })
                    .collect::<HashMap<_, _>>();

                return Ok(BatchExecutionResult {
                    translations,
                    attempts: attempt + 1,
                });
            }
            Err(error) => {
                last_error = Some(UserFacingError::new(
                    "Translation Request Failed",
                    format!(
                        "Batch {} for model '{}' failed after request attempt {}.",
                        batch.batch_index + 1,
                        batch.runtime.model,
                        attempt + 1
                    ),
                    Some(error.to_string()),
                ));
            }
        }
    }

    Err(last_error.unwrap_or_else(|| {
        UserFacingError::new(
            "Translation Request Failed",
            "The provider request failed for an unknown reason.",
            None,
        )
    }))
}

fn parse_translations(response: &str) -> HashMap<usize, String> {
    let thinking_regex = Regex::new(r"(?s)<thinking>.*?</thinking>").ok();
    let cleaned = thinking_regex
        .as_ref()
        .map(|regex| regex.replace_all(response, "").to_string())
        .unwrap_or_else(|| response.to_string());
    let line_regex = Regex::new(r"^(\d+)[\.\)）、\s]\s*(.*)?$").ok();

    let mut translations = HashMap::new();
    let mut current_index: Option<usize> = None;
    let mut current_lines: Vec<String> = vec![];

    for raw_line in cleaned.lines() {
        let line = raw_line.trim();
        if line.is_empty() {
            continue;
        }

        if line == "---" {
            if let Some(index) = current_index.take() {
                if !current_lines.is_empty() {
                    translations.insert(index, current_lines.join("\n").trim().to_string());
                }
                current_lines.clear();
            }
            continue;
        }

        if let Some(regex) = &line_regex {
            if let Some(captures) = regex.captures(line) {
                if let Some(index) = current_index.take() {
                    if !current_lines.is_empty() {
                        translations.insert(index, current_lines.join("\n").trim().to_string());
                    }
                    current_lines.clear();
                }

                current_index = captures
                    .get(1)
                    .and_then(|value| value.as_str().parse::<usize>().ok())
                    .map(|value| value.saturating_sub(1));
                if let Some(text) = captures.get(2).map(|value| value.as_str().trim()) {
                    if !text.is_empty() {
                        current_lines.push(text.to_string());
                    }
                }
                continue;
            }
        }

        if current_index.is_some() {
            current_lines.push(line.to_string());
        }
    }

    if let Some(index) = current_index {
        if !current_lines.is_empty() {
            translations.insert(index, current_lines.join("\n").trim().to_string());
        }
    }

    translations
}
