use crate::domain::{AgentRecord, AgentType};
use anyhow::Result;
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use std::path::PathBuf;
use std::sync::Mutex;

pub struct CacheDb {
    conn: Mutex<Connection>,
}

impl CacheDb {
    pub fn new(db_path: Option<PathBuf>) -> Result<Self> {
        let path = db_path.unwrap_or_else(|| {
            let home = std::env::var("HOME")
                .or_else(|_| std::env::var("USERPROFILE"))
                .unwrap_or_else(|_| ".".to_string());
            PathBuf::from(home)
                .join(".agent-insights")
                .join("cache.db")
        });

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(&path)?;
        let db = Self { conn: Mutex::new(conn) };
        db.init()?;
        tracing::info!("Cache database initialized at {:?}", path);
        Ok(db)
    }

    fn init(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "CREATE TABLE IF NOT EXISTS file_cache (
                file_path TEXT PRIMARY KEY,
                agent_type TEXT NOT NULL,
                created_at TEXT NOT NULL,
                modified_at TEXT NOT NULL,
                file_size INTEGER NOT NULL,
                session_id TEXT,
                tokens_input INTEGER,
                tokens_output INTEGER,
                tokens_cached INTEGER,
                tokens_reasoning INTEGER,
                tokens_total INTEGER,
                tool_calls TEXT,
                cached_at TEXT NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_agent_type ON file_cache(agent_type)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_modified_at ON file_cache(modified_at)",
            [],
        )?;

        // Migrate existing tables to add tool_calls column if it doesn't exist
        conn.execute(
            "ALTER TABLE file_cache ADD COLUMN tool_calls TEXT",
            [],
        ).ok(); // Ignore error if column already exists

        // Migrate existing tables to add tokens_reasoning column if it doesn't exist
        conn.execute(
            "ALTER TABLE file_cache ADD COLUMN tokens_reasoning INTEGER",
            [],
        ).ok(); // Ignore error if column already exists

        Ok(())
    }

    pub fn get_cached_record(&self, file_path: &str, modified_at: &DateTime<Utc>) -> Result<Option<AgentRecord>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT agent_type, created_at, modified_at, file_size, session_id,
                    tokens_input, tokens_output, tokens_cached, tokens_reasoning, tokens_total, tool_calls
             FROM file_cache
             WHERE file_path = ?1 AND modified_at = ?2"
        )?;

        let modified_str = modified_at.to_rfc3339();
        let result = stmt.query_row(params![file_path, modified_str], |row| {
            let agent_type_str: String = row.get(0)?;
            let agent_type = match agent_type_str.as_str() {
                "Claude" => AgentType::Claude,
                "Gemini" => AgentType::Gemini,
                "Codex" => AgentType::Codex,
                _ => AgentType::Codexia,
            };

            let created_str: String = row.get(1)?;
            let modified_str: String = row.get(2)?;

            let tokens = if let (Some(input), Some(output), Some(cached), Some(reasoning), Some(total)) = (
                row.get::<_, Option<i64>>(5)?,
                row.get::<_, Option<i64>>(6)?,
                row.get::<_, Option<i64>>(7)?,
                row.get::<_, Option<i64>>(8)?,
                row.get::<_, Option<i64>>(9)?,
            ) {
                Some(crate::domain::TokenInfo {
                    input: input as u64,
                    output: output as u64,
                    cached: cached as u64,
                    cache_creation: 0,
                    reasoning: reasoning as u64,
                    total: total as u64,
                })
            } else {
                None
            };

            let tool_calls: Vec<String> = row.get::<_, Option<String>>(10)?
                .and_then(|json_str| serde_json::from_str(&json_str).ok())
                .unwrap_or_default();

            Ok(AgentRecord {
                agent_type,
                file_path: file_path.to_string(),
                created_at: created_str.parse().unwrap_or_else(|_| Utc::now()),
                modified_at: modified_str.parse().unwrap_or_else(|_| Utc::now()),
                file_size: row.get::<_, i64>(3)? as u64,
                session_id: row.get(4)?,
                tokens,
                tool_calls,
            })
        });

        match result {
            Ok(record) => Ok(Some(record)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn cache_record(&self, record: &AgentRecord) -> Result<()> {
        let agent_type_str = format!("{:?}", record.agent_type);
        let created_str = record.created_at.to_rfc3339();
        let modified_str = record.modified_at.to_rfc3339();
        let cached_at = Utc::now().to_rfc3339();

        let (tokens_input, tokens_output, tokens_cached, tokens_reasoning, tokens_total) = if let Some(ref tokens) = record.tokens {
            (
                Some(tokens.input as i64),
                Some(tokens.output as i64),
                Some(tokens.cached as i64),
                Some(tokens.reasoning as i64),
                Some(tokens.total as i64),
            )
        } else {
            (None, None, None, None, None)
        };

        let tool_calls_json = serde_json::to_string(&record.tool_calls)?;

        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO file_cache
             (file_path, agent_type, created_at, modified_at, file_size, session_id,
              tokens_input, tokens_output, tokens_cached, tokens_reasoning, tokens_total, tool_calls, cached_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            params![
                record.file_path,
                agent_type_str,
                created_str,
                modified_str,
                record.file_size as i64,
                record.session_id,
                tokens_input,
                tokens_output,
                tokens_cached,
                tokens_reasoning,
                tokens_total,
                tool_calls_json,
                cached_at,
            ],
        )?;

        Ok(())
    }

    pub fn get_cache_stats(&self) -> Result<CacheStats> {
        let conn = self.conn.lock().unwrap();
        let total_entries: i64 = conn.query_row(
            "SELECT COUNT(*) FROM file_cache",
            [],
            |row| row.get(0)
        )?;

        let mut stmt = conn.prepare(
            "SELECT agent_type, COUNT(*) FROM file_cache GROUP BY agent_type"
        )?;

        let mut by_agent = std::collections::HashMap::new();
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })?;

        for row in rows {
            let (agent, count) = row?;
            by_agent.insert(agent, count as usize);
        }

        Ok(CacheStats {
            total_entries: total_entries as usize,
            entries_by_agent: by_agent,
        })
    }
}

#[derive(Debug)]
pub struct CacheStats {
    pub total_entries: usize,
    pub entries_by_agent: std::collections::HashMap<String, usize>,
}
