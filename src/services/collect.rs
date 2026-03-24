#[cfg(feature = "cache")]
use crate::cache::CacheDb;
use crate::agents::{ClaudeScanner, GeminiScanner, CodexScanner};
use crate::domain::AgentRecord;
use anyhow::Result;
#[cfg(feature = "cache")]
use std::sync::Arc;

pub struct CollectionService {
    home_dir: String,
    #[cfg(feature = "cache")]
    cache: Arc<CacheDb>,
}

impl CollectionService {
    pub fn new() -> Result<Self> {
        let home_dir = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .unwrap_or_else(|_| ".".to_string());

        #[cfg(feature = "cache")]
        let cache = Arc::new(CacheDb::new(None)?);

        Ok(Self {
            home_dir,
            #[cfg(feature = "cache")]
            cache,
        })
    }

    #[cfg(feature = "cache")]
    pub fn cache_stats(&self) -> Result<crate::cache::db::CacheStats> {
        self.cache.get_cache_stats()
    }

    pub async fn collect_all(&self) -> Result<Vec<AgentRecord>> {
        let mut all_records = Vec::new();

        #[cfg(feature = "cache")]
        let claude = ClaudeScanner::with_cache(&self.home_dir, Arc::clone(&self.cache));
        #[cfg(not(feature = "cache"))]
        let claude = ClaudeScanner::new(&self.home_dir);

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

    /// Refresh the cache by scanning all files, then return records with created_at >= since.
    /// `since` is an ISO date "YYYY-MM-DD"; None returns all records.
    #[cfg(feature = "cache")]
    pub async fn collect_since(&self, since: Option<&str>) -> Result<Vec<AgentRecord>> {
        let _ = self.collect_all().await; // populate cache
        self.cache.get_all_records(since)
    }

    pub async fn collect_claude(&self) -> Result<Vec<AgentRecord>> {
        #[cfg(feature = "cache")]
        let scanner = ClaudeScanner::with_cache(&self.home_dir, Arc::clone(&self.cache));
        #[cfg(not(feature = "cache"))]
        let scanner = ClaudeScanner::new(&self.home_dir);
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
