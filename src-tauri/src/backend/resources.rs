use std::fs;
use std::path::Path;

use crate::backend::error::UserFacingError;
use crate::backend::models::TerminologyDictionary;
pub fn read_text_file(path: &Path) -> Result<String, UserFacingError> {
    fs::read_to_string(path).map_err(|error| UserFacingError::io("Read", path, &error))
}

pub fn write_text_file(path: &Path, content: &str) -> Result<(), UserFacingError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| UserFacingError::io("Create directory", parent, &error))?;
    }
    fs::write(path, content).map_err(|error| UserFacingError::io("Write", path, &error))
}

pub fn read_terminology(path: &Path) -> Result<TerminologyDictionary, UserFacingError> {
    let content = read_text_file(path)?;
    serde_json::from_str(&content).map_err(|error| UserFacingError::invalid_json(path, error))
}

pub fn write_terminology(path: &Path, payload: &TerminologyDictionary) -> Result<(), UserFacingError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|error| UserFacingError::io("Create directory", parent, &error))?;
    }
    let content = serde_json::to_string_pretty(payload).map_err(|error| {
        UserFacingError::new(
            "Serialization Error",
            format!("Failed to serialize {}", path.display()),
            Some(error.to_string()),
        )
    })?;
    std::fs::write(path, content).map_err(|error| UserFacingError::io("Write", path, &error))
}
