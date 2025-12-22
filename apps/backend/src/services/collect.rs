use crate::agents::{ClaudeScanner, GeminiScanner, CodexScanner};
use crate::cache::CacheDb;
use crate::domain::AgentRecord;
use anyhow::Result;
use std::sync::Arc;

pub struct CollectionService {
    home_dir: String,
    cache: Arc<CacheDb>,
}

impl CollectionService {
    pub fn new() -> Result<Self> {
        let home_dir = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .unwrap_or_else(|_| ".".to_string());

        let cache = Arc::new(CacheDb::new(None)?);

        Ok(Self { home_dir, cache })
    }

    pub fn cache_stats(&self) -> Result<crate::cache::db::CacheStats> {
        self.cache.get_cache_stats()
    }

    pub async fn collect_all(&self) -> Result<Vec<AgentRecord>> {
        let mut all_records = Vec::new();

        let claude = ClaudeScanner::with_cache(&self.home_dir, Arc::clone(&self.cache));
        if let Ok(records) = claude.scan() {
            tracing::info!("Collected {} Claude records", records.len());
            all_records.extend(records);
        }

        let gemini = GeminiScanner::new(&self.home_dir);
        if let Ok(records) = gemini.scan() {
            tracing::info!("Collected {} Gemini records", records.len());
            all_records.extend(records);
        }

        let codex = CodexScanner::new(&self.home_dir);
        if let Ok(records) = codex.scan() {
            tracing::info!("Collected {} Codex records", records.len());
            all_records.extend(records);
        }

        tracing::info!("Total collected: {} records", all_records.len());
        Ok(all_records)
    }

    pub async fn collect_claude(&self) -> Result<Vec<AgentRecord>> {
        let scanner = ClaudeScanner::with_cache(&self.home_dir, Arc::clone(&self.cache));
        scanner.scan()
    }

    pub async fn collect_gemini(&self) -> Result<Vec<AgentRecord>> {
        let scanner = GeminiScanner::new(&self.home_dir);
        scanner.scan()
    }

    pub async fn collect_codex(&self) -> Result<Vec<AgentRecord>> {
        let scanner = CodexScanner::new(&self.home_dir);
        scanner.scan()
    }
}
