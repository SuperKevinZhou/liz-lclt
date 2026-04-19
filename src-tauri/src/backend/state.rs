use std::path::PathBuf;
use std::time::Instant;

use crate::backend::models::TranslationTask;

#[derive(Debug, Default)]
pub struct AppState {
    pub workspace_root: Option<PathBuf>,
    pub task: Option<TranslationTaskRecord>,
}

#[derive(Debug, Clone)]
pub struct TranslationTaskRecord {
    pub task: TranslationTask,
    pub started_at: Instant,
    pub cancelled: bool,
}
