use crate::domain::{AgentRecord, AgentType, TokenInfo};
use crate::scanner::{FileInfo, FileScanner};
use anyhow::Result;
use serde_json::Value;
use std::fs;
use std::path::PathBuf;

pub struct CodexScanner {
    scanner: FileScanner,
}

impl CodexScanner {
    pub fn new(home_dir: &str) -> Self {
        let root = PathBuf::from(home_dir).join(".codex");
        Self {
            scanner: FileScanner::new(root),
        }
    }

    pub fn scan(&self) -> Result<Vec<AgentRecord>> {
        let jsonl_files = self.scanner.scan_jsonl_files()?;

        let mut records = Vec::new();

        for file_info in jsonl_files.iter() {
            if let Ok(record) = self.parse_jsonl_file(file_info) {
                records.push(record);
            }
        }

        Ok(records)
    }

    fn parse_jsonl_file(&self, file_info: &FileInfo) -> Result<AgentRecord> {
        let content = fs::read_to_string(&file_info.path)?;

        let mut session_id: Option<String> = None;
        let mut total_input = 0u64;
        let mut total_cached_input = 0u64;
        let mut total_output = 0u64;
        let mut total_reasoning = 0u64;
        let mut total_tokens = 0u64;
        let mut tool_calls: Vec<String> = Vec::new();

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            if let Ok(json) = serde_json::from_str::<Value>(line) {
                // Extract data based on event type
                if let Some(event_type) = json.get("type").and_then(|t| t.as_str()) {
                    if let Some(payload) = json.get("payload") {
                        // Extract token usage from event_msg -> token_count
                        if event_type == "event_msg" {
                            if let Some(payload_type) = payload.get("type").and_then(|t| t.as_str()) {
                                if payload_type == "token_count" {
                                    if let Some(info) = payload.get("info") {
                                        if let Some(total_usage) = info.get("total_token_usage") {
                                            if let Some(input) = total_usage.get("input_tokens").and_then(|v| v.as_u64()) {
                                                total_input = input;
                                            }
                                            if let Some(cached) = total_usage.get("cached_input_tokens").and_then(|v| v.as_u64()) {
                                                total_cached_input = cached;
                                            }
                                            if let Some(output) = total_usage.get("output_tokens").and_then(|v| v.as_u64()) {
                                                total_output = output;
                                            }
                                            if let Some(reasoning) = total_usage.get("reasoning_output_tokens").and_then(|v| v.as_u64()) {
                                                total_reasoning = reasoning;
                                            }
                                            if let Some(total) = total_usage.get("total_tokens").and_then(|v| v.as_u64()) {
                                                total_tokens = total;
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        // Extract tool calls from response_item -> custom_tool_call
                        if event_type == "response_item" {
                            if let Some(payload_type) = payload.get("type").and_then(|t| t.as_str()) {
                                if payload_type == "custom_tool_call" {
                                    if let Some(tool_name) = payload.get("name").and_then(|t| t.as_str()) {
                                        tool_calls.push(tool_name.to_string());
                                    }
                                }
                            }
                        }
                    }
                }

                // Extract session ID from session_meta
                if session_id.is_none() {
                    if let Some(event_type) = json.get("type").and_then(|t| t.as_str()) {
                        if event_type == "session_meta" {
                            if let Some(payload) = json.get("payload") {
                                if let Some(id) = payload.get("id").and_then(|s| s.as_str()) {
                                    session_id = Some(id.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }

        let tokens = if total_input > 0 || total_output > 0 {
            Some(TokenInfo {
                input: total_input,
                output: total_output,
                cached: total_cached_input,
                cache_creation: 0,
                reasoning: total_reasoning,
                total: total_tokens,
            })
        } else {
            None
        };

        Ok(AgentRecord {
            agent_type: AgentType::Codex,
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
