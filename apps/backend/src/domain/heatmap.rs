use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DayActivity {
    pub date: String,
    pub count: usize,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallStats {
    pub tool_name: String,
    pub count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenStats {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_creation_tokens: u64,
    pub cache_read_tokens: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_tokens: Option<u64>,
    pub total_tokens: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeatmapData {
    pub agent: String,
    pub data: Vec<DayActivity>,
    pub max_count: usize,
    pub total_files: usize,
    pub total_size: u64,
    pub tool_calls: Vec<ToolCallStats>,
    pub token_stats: TokenStats,
}

impl HeatmapData {
    pub fn from_counts(
        agent: String,
        day_counts: HashMap<NaiveDate, (usize, u64)>,
        tool_calls: Vec<ToolCallStats>,
        token_stats: TokenStats,
    ) -> Self {
        let mut data: Vec<DayActivity> = day_counts
            .into_iter()
            .map(|(date, (count, size))| DayActivity {
                date: date.format("%Y-%m-%d").to_string(),
                count,
                size,
            })
            .collect();

        data.sort_by(|a, b| a.date.cmp(&b.date));

        let max_count = data.iter().map(|d| d.count).max().unwrap_or(0);
        let total_files = data.iter().map(|d| d.count).sum();
        let total_size = data.iter().map(|d| d.size).sum();

        HeatmapData {
            agent,
            data,
            max_count,
            total_files,
            total_size,
            tool_calls,
            token_stats,
        }
    }
}
