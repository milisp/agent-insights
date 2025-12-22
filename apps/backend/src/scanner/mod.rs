use anyhow::Result;
use chrono::{DateTime, Utc};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Clone)]
pub struct FileInfo {
    pub path: PathBuf,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
    pub size: u64,
}

pub struct FileScanner {
    pub root: PathBuf,
}

impl FileScanner {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    pub fn scan_json_files(&self) -> Result<Vec<FileInfo>> {
        self.scan_files_with_extension("json")
    }

    pub fn scan_jsonl_files(&self) -> Result<Vec<FileInfo>> {
        self.scan_files_with_extension("jsonl")
    }

    pub fn scan_files_with_extension(&self, ext: &str) -> Result<Vec<FileInfo>> {
        let mut files = Vec::new();

        if !self.root.exists() {
            tracing::warn!("Directory does not exist: {:?}", self.root);
            return Ok(files);
        }

        for entry in WalkDir::new(&self.root)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }

            if let Some(file_ext) = path.extension() {
                if file_ext == ext {
                    if let Ok(file_info) = self.get_file_info(path) {
                        files.push(file_info);
                    }
                }
            }
        }

        Ok(files)
    }

    fn get_file_info(&self, path: &Path) -> Result<FileInfo> {
        let metadata = fs::metadata(path)?;
        let modified = metadata.modified()?;
        let modified_at: DateTime<Utc> = modified.into();

        let created = metadata.created().unwrap_or(modified);
        let created_at: DateTime<Utc> = created.into();

        Ok(FileInfo {
            path: path.to_path_buf(),
            created_at,
            modified_at,
            size: metadata.len(),
        })
    }
}
