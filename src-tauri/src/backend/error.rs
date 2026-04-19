use std::fmt::{Display, Formatter};
use std::io;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UserFacingError {
    pub title: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

impl UserFacingError {
    pub fn new(
        title: impl Into<String>,
        message: impl Into<String>,
        details: Option<String>,
    ) -> Self {
        Self {
            title: title.into(),
            message: message.into(),
            details,
        }
    }

    pub fn io(action: &str, path: &std::path::Path, error: &io::Error) -> Self {
        Self::new(
            "File System Error",
            format!("{action} failed for {}", path.display()),
            Some(error.to_string()),
        )
    }

    pub fn invalid_json(path: &std::path::Path, error: impl Display) -> Self {
        Self::new(
            "Invalid JSON",
            format!("Failed to parse {}", path.display()),
            Some(error.to_string()),
        )
    }
}

impl Display for UserFacingError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.title, self.message)
    }
}

impl std::error::Error for UserFacingError {}
