use std::path::PathBuf;

use claw_rag_service::{EmbedConfig, MemoryService, SqliteWindMemory};

#[derive(Clone)]
pub(crate) struct AppState {
    pub(crate) db_path: PathBuf,
    pub(crate) client: reqwest::Client,
    pub(crate) cfg: EmbedConfig,
    pub(crate) memory: SqliteWindMemory,
    pub(crate) json_memory: MemoryService,
}
