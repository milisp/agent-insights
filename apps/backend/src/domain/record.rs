use chrono::{DateTime, Utc, NaiveDate};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentType {
    Claude,
    Codex,
    Gemini,
    Codexia,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRecord {
    pub agent_type: AgentType,
    pub file_path: String,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
    pub file_size: u64,
    pub session_id: Option<String>,
    pub tokens: Option<TokenInfo>,
    pub tool_calls: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenInfo {
    pub input: u64,
    pub output: u64,
    pub cached: u64,
    pub cache_creation: u64,
    pub reasoning: u64,
    pub total: u64,
}

impl AgentRecord {
    pub fn date(&self) -> NaiveDate {
        self.created_at.date_naive()
    }
}
