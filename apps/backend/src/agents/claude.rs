use crate::cache::CacheDb;
use crate::domain::{AgentRecord, AgentType, TokenInfo};
use crate::scanner::{FileInfo, FileScanner};
use anyhow::Result;
use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

pub struct ClaudeScanner {
    scanner: FileScanner,
    cache: Option<Arc<CacheDb>>,
}

impl ClaudeScanner {
    pub fn new(home_dir: &str) -> Self {
        let root = PathBuf::from(home_dir)
            .join(".claude")
            .join("projects");
        Self {
            scanner: FileScanner::new(root),
            cache: None,
        }
    }

    pub fn with_cache(home_dir: &str, cache: Arc<CacheDb>) -> Self {
        let root = PathBuf::from(home_dir)
            .join(".claude")
            .join("projects");
        Self {
            scanner: FileScanner::new(root),
            cache: Some(cache),
        }
    }

    pub fn scan(&self) -> Result<Vec<AgentRecord>> {
        let files = self.scanner.scan_jsonl_files()?;
        let mut records = Vec::new();
        let mut cache_hits = 0;
        let mut cache_misses = 0;

        for file_info in files {
            if let Some(file_name) = file_info.path.file_name().and_then(|n| n.to_str()) {
                if file_name.starts_with("agent") {
                    continue;
                }
            }
            let record = if let Some(ref cache) = self.cache {
                if let Ok(Some(cached)) = cache.get_cached_record(&file_info.path.to_string_lossy(), &file_info.modified_at) {
                    cache_hits += 1;
                    cached
                } else {
                    cache_misses += 1;
                    let record = self.parse_jsonl_file(&file_info)?;
                    let _ = cache.cache_record(&record);
                    record
                }
            } else {
                self.parse_jsonl_file(&file_info)?
            };

            records.push(record);
        }

        if let Some(_) = self.cache {
            tracing::debug!("Claude scan: {} cache hits, {} cache misses", cache_hits, cache_misses);
        }

        Ok(records)
    }

    fn parse_jsonl_file(&self, file_info: &FileInfo) -> Result<AgentRecord> {
        let content = fs::read_to_string(&file_info.path)?;

        let mut session_id: Option<String> = None;
        let mut total_input = 0u64;
        let mut total_output = 0u64;
        let mut total_cached = 0u64;
        let mut total_cache_creation = 0u64;
        let mut tool_calls: Vec<String> = Vec::new();

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            if let Ok(json) = serde_json::from_str::<Value>(line) {
                if session_id.is_none() {
                    if let Some(sid) = json.get("sessionId")
                        .or_else(|| json.get("session_id"))
                        .and_then(|v| v.as_str()) {
                        session_id = Some(sid.to_string());
                    }
                }

                // Extract tool_use information
                if let Some(message) = json.get("message") {
                    if let Some(content) = message.get("content").and_then(|c| c.as_array()) {
                        for item in content {
                            if let Some(typ) = item.get("type").and_then(|t| t.as_str()) {
                                if typ == "tool_use" {
                                    if let Some(name) = item.get("name").and_then(|n| n.as_str()) {
                                        tool_calls.push(name.to_string());
                                    }
                                }
                            }
                        }
                    }
                }

                // Extract usage information
                if let Some(usage) = json.get("usage").or_else(|| json.get("message").and_then(|m| m.get("usage"))) {
                    if let Some(input) = usage.get("input_tokens").and_then(|v| v.as_u64()) {
                        total_input = total_input.saturating_add(input);
                    }
                    if let Some(output) = usage.get("output_tokens").and_then(|v| v.as_u64()) {
                        total_output = total_output.saturating_add(output);
                    }
                    if let Some(cached) = usage.get("cache_read_input_tokens").and_then(|v| v.as_u64()) {
                        total_cached = total_cached.saturating_add(cached);
                    }
                    if let Some(cache_creation) = usage.get("cache_creation_input_tokens").and_then(|v| v.as_u64()) {
                        total_cache_creation = total_cache_creation.saturating_add(cache_creation);
                    }
                }
            }
        }

        let tokens = if total_input > 0 || total_output > 0 {
            Some(TokenInfo {
                input: total_input,
                output: total_output,
                cached: total_cached,
                cache_creation: total_cache_creation,
                total: total_input + total_output + total_cached + total_cache_creation,
            })
        } else {
            None
        };

        Ok(AgentRecord {
            agent_type: AgentType::Claude,
            file_path: file_info.path.to_string_lossy().to_string(),
            created_at: file_info.created_at,
            modified_at: file_info.modified_at,
            file_size: file_info.size,
            session_id,
            tokens,
            tool_calls,
        })
    }
}
