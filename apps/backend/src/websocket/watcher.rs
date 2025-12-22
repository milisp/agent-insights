use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::PathBuf;

use super::{UpdateMessage, UpdateSender};

pub struct FileWatcher {
    tx: UpdateSender,
    watcher: Option<RecommendedWatcher>,
}

impl FileWatcher {
    pub fn new(tx: UpdateSender) -> Self {
        Self { tx, watcher: None }
    }

    pub fn start(&mut self, paths: Vec<(String, PathBuf)>) -> notify::Result<()> {
        let tx = self.tx.clone();

        let watcher = RecommendedWatcher::new(
            move |res: Result<Event, notify::Error>| {
                if let Ok(event) = res {
                    handle_event(event, &tx);
                }
            },
            Config::default(),
        )?;

        self.watcher = Some(watcher);

        for (agent, path) in paths {
            if path.exists() {
                if let Some(ref mut w) = self.watcher {
                    if let Err(e) = w.watch(&path, RecursiveMode::Recursive) {
                        tracing::warn!("Failed to watch {:?} for {}: {}", path, agent, e);
                    } else {
                        tracing::info!("Watching {:?} for {} changes", path, agent);
                    }
                }
            }
        }

        Ok(())
    }
}

fn handle_event(event: Event, tx: &UpdateSender) {
    use notify::EventKind;

    match event.kind {
        EventKind::Create(_) | EventKind::Modify(_) => {
            for path in event.paths {
                if let Some(ext) = path.extension() {
                    if ext == "json" || ext == "jsonl" {
                        if let Some(agent) = detect_agent_from_path(&path) {
                            let msg = UpdateMessage::FileAdded {
                                agent,
                                file_path: path.to_string_lossy().to_string(),
                            };
                            let _ = tx.send(msg);
                        }
                    }
                }
            }
        }
        _ => {}
    }
}

fn detect_agent_from_path(path: &PathBuf) -> Option<String> {
    let path_str = path.to_string_lossy();
    if path_str.contains("/.claude/") {
        Some("Claude".to_string())
    } else if path_str.contains("/.gemini/") {
        Some("Gemini".to_string())
    } else if path_str.contains("/.codex/") {
        Some("Codex".to_string())
    } else {
        None
    }
}
