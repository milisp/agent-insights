use crate::domain::{AgentRecord, AgentType, TokenInfo};
use crate::scanner::{FileInfo, FileScanner};
use anyhow::Result;
use serde_json::Value;
use std::fs;
use std::path::PathBuf;

pub struct GeminiScanner {
    scanner: FileScanner,
}

impl GeminiScanner {
    pub fn new(home_dir: &str) -> Self {
        let root = PathBuf::from(home_dir)
            .join(".gemini")
            .join("tmp");
        Self {
            scanner: FileScanner::new(root),
        }
    }

    pub fn scan(&self) -> Result<Vec<AgentRecord>> {
        let files = self.scanner.scan_json_files()?;
        let mut records = Vec::new();

        for file_info in files {
            if file_info.path.to_string_lossy().contains("/chats/") {
                if let Ok(record) = self.parse_chat_file(&file_info) {
                    records.push(record);
                }
            }
        }

        Ok(records)
    }

    fn parse_chat_file(&self, file_info: &FileInfo) -> Result<AgentRecord> {
        let content = fs::read_to_string(&file_info.path)?;
        let json: Value = serde_json::from_str(&content)?;

        let session_id = json.get("sessionId")
            .or_else(|| json.get("session_id"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let mut total_input = 0u64;
        let mut total_output = 0u64;
        let mut total_cached = 0u64;
        let mut total_thoughts = 0u64;
        let mut total_tool = 0u64;
        let mut tool_calls: Vec<String> = Vec::new();

        if let Some(messages) = json.get("messages").and_then(|v| v.as_array()) {
            for msg in messages {
                // Extract token usage from tokens field
                if let Some(tokens) = msg.get("tokens") {
                    if let Some(input) = tokens.get("input").and_then(|v| v.as_u64()) {
                        total_input = total_input.saturating_add(input);
                    }
                    if let Some(output) = tokens.get("output").and_then(|v| v.as_u64()) {
                        total_output = total_output.saturating_add(output);
                    }
                    if let Some(cached) = tokens.get("cached").and_then(|v| v.as_u64()) {
                        total_cached = total_cached.saturating_add(cached);
                    }
                    if let Some(thoughts) = tokens.get("thoughts").and_then(|v| v.as_u64()) {
                        total_thoughts = total_thoughts.saturating_add(thoughts);
                    }
                    if let Some(tool) = tokens.get("tool").and_then(|v| v.as_u64()) {
                        total_tool = total_tool.saturating_add(tool);
                    }
                }

                // Extract tool calls
                if let Some(tool_calls_array) = msg.get("toolCalls").and_then(|v| v.as_array()) {
                    for tool_call in tool_calls_array {
                        if let Some(name) = tool_call.get("name").and_then(|n| n.as_str()) {
                            tool_calls.push(name.to_string());
                        }
                    }
                }
            }
        }

        let tokens = if total_input > 0 || total_output > 0 {
            // Gemini: aggregate output = output + thoughts + tool (like CodMate does)
            let aggregated_output = total_output + total_thoughts + total_tool;
            Some(TokenInfo {
                input: total_input,
                output: aggregated_output,
                cached: total_cached,
                cache_creation: 0,
                total: total_input + aggregated_output + total_cached,
            })
        } else {
            None
        };

        Ok(AgentRecord {
            agent_type: AgentType::Gemini,
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
