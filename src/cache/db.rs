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
                cached_at TEXT NOT NULL,
                cwd TEXT
            )",
            [],
        )?;

        Ok(())
    }

    pub fn get_cached_record(&self, file_path: &str, modified_at: &DateTime<Utc>) -> Result<Option<AgentRecord>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT agent_type, created_at, modified_at, file_size, session_id,
                    tokens_input, tokens_output, tokens_cached, tokens_reasoning, tokens_total, tool_calls, model, cwd
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
                model: row.get(11)?,
                cwd: row.get(12)?,
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
              tokens_input, tokens_output, tokens_cached, tokens_reasoning, tokens_total, tool_calls, cached_at, model, cwd)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
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
                record.model,
                record.cwd,
            ],
        )?;

        Ok(())
    }

    /// Return all cached records, optionally filtered to created_at >= since (ISO date "YYYY-MM-DD").
    pub fn get_all_records(&self, since: Option<&str>) -> Result<Vec<AgentRecord>> {
        let conn = self.conn.lock().unwrap();

        let sql_with    = "SELECT file_path, agent_type, created_at, modified_at, file_size, session_id,
                                  tokens_input, tokens_output, tokens_cached, tokens_reasoning, tokens_total, tool_calls, model, cwd
                           FROM file_cache WHERE created_at >= ?1 ORDER BY created_at";
        let sql_all     = "SELECT file_path, agent_type, created_at, modified_at, file_size, session_id,
                                  tokens_input, tokens_output, tokens_cached, tokens_reasoning, tokens_total, tool_calls, model, cwd
                           FROM file_cache ORDER BY created_at";

        let mut stmt = conn.prepare(if since.is_some() { sql_with } else { sql_all })?;

        fn parse_row(row: &rusqlite::Row) -> rusqlite::Result<AgentRecord> {
            let agent_type = match row.get::<_, String>(1)?.as_str() {
                "Claude" => AgentType::Claude,
                "Gemini" => AgentType::Gemini,
                "Codex"  => AgentType::Codex,
                _        => AgentType::Codexia,
            };
            let tokens = match (
                row.get::<_, Option<i64>>(6)?,
                row.get::<_, Option<i64>>(7)?,
                row.get::<_, Option<i64>>(8)?,
                row.get::<_, Option<i64>>(9)?,
                row.get::<_, Option<i64>>(10)?,
            ) {
                (Some(i), Some(o), Some(c), Some(r), Some(t)) => Some(crate::domain::TokenInfo {
                    input: i as u64, output: o as u64, cached: c as u64,
                    cache_creation: 0, reasoning: r as u64, total: t as u64,
                }),
                _ => None,
            };
            let tool_calls: Vec<String> = row.get::<_, Option<String>>(11)?
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default();
            let created_str: String = row.get(2)?;
            let modified_str: String = row.get(3)?;
            Ok(AgentRecord {
                agent_type,
                file_path:  row.get(0)?,
                created_at: created_str.parse().unwrap_or_else(|_| chrono::Utc::now()),
                modified_at: modified_str.parse().unwrap_or_else(|_| chrono::Utc::now()),
                file_size:  row.get::<_, i64>(4)? as u64,
                session_id: row.get(5)?,
                model:      row.get(12)?,
                cwd:        row.get(13)?,
                tokens,
                tool_calls,
            })
        }

        let records: rusqlite::Result<Vec<AgentRecord>> = if let Some(s) = since {
            stmt.query_map(params![s], parse_row)?.collect()
        } else {
            stmt.query_map([], parse_row)?.collect()
        };
        Ok(records?)
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
