use crate::domain::{AgentRecord, HeatmapData, ToolCallStats, TokenStats};
use chrono::NaiveDate;
use std::collections::HashMap;

pub struct AggregationService;

impl AggregationService {
    pub fn aggregate_by_date(records: Vec<AgentRecord>) -> HeatmapData {
        let mut day_counts: HashMap<NaiveDate, (usize, u64)> = HashMap::new();
        let mut tool_call_counts: HashMap<String, usize> = HashMap::new();
        let mut total_input = 0u64;
        let mut total_output = 0u64;
        let mut total_cache_creation = 0u64;
        let mut total_cache_read = 0u64;

        for record in &records {
            let date = record.date();
            let entry = day_counts.entry(date).or_insert((0, 0));
            entry.0 += 1;
            entry.1 += record.file_size;

            // Aggregate tool calls
            for tool in &record.tool_calls {
                *tool_call_counts.entry(tool.clone()).or_insert(0) += 1;
            }

            // Aggregate tokens
            if let Some(ref tokens) = record.tokens {
                total_input += tokens.input;
                total_output += tokens.output;
                total_cache_creation += tokens.cache_creation;
                total_cache_read += tokens.cached;
            }
        }

        let agent = if !records.is_empty() {
            format!("{:?}", records[0].agent_type)
        } else {
            "Unknown".to_string()
        };

        let mut tool_calls: Vec<ToolCallStats> = tool_call_counts
            .into_iter()
            .map(|(tool_name, count)| ToolCallStats { tool_name, count })
            .collect();
        tool_calls.sort_by(|a, b| b.count.cmp(&a.count));

        let token_stats = TokenStats {
            input_tokens: total_input,
            output_tokens: total_output,
            cache_creation_tokens: total_cache_creation,
            cache_read_tokens: total_cache_read,
            total_tokens: total_input + total_output + total_cache_creation + total_cache_read,
        };

        HeatmapData::from_counts(agent, day_counts, tool_calls, token_stats)
    }

    pub fn aggregate_by_agent(records: Vec<AgentRecord>) -> HashMap<String, HeatmapData> {
        let mut by_agent: HashMap<String, Vec<AgentRecord>> = HashMap::new();

        for record in records {
            let agent_key = format!("{:?}", record.agent_type);
            by_agent.entry(agent_key).or_default().push(record);
        }

        by_agent
            .into_iter()
            .map(|(agent, records)| {
                let heatmap = Self::aggregate_by_date(records);
                (agent, heatmap)
            })
            .collect()
    }
}
