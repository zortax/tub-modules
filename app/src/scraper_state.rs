use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

// Re-export types from scraper_types
pub use crate::scraper_types::{LogLevel, ScraperEvent, ScraperStatus, ScrapingRun};

/// Maximum number of log entries to keep in memory
const MAX_LOG_ENTRIES: usize = 1000;

/// Shared state for the scraper
#[derive(Clone)]
pub struct ScraperState {
    inner: Arc<RwLock<ScraperStateInner>>,
    event_tx: broadcast::Sender<ScraperEvent>,
}

use serde::{Deserialize, Serialize};

struct ScraperStateInner {
    pub current_run: Option<ScrapingRun>,
    pub csv_content: Option<String>,
    pub csv_info: Option<CsvInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CsvInfo {
    pub total_rows: usize,
    pub valid_modules: usize,
    pub invalid_rows: usize,
    pub file_name: String,
}

impl ScraperState {
    pub fn new() -> Self {
        let (event_tx, _) = broadcast::channel(100);
        Self {
            inner: Arc::new(RwLock::new(ScraperStateInner {
                current_run: None,
                csv_content: None,
                csv_info: None,
            })),
            event_tx,
        }
    }

    pub async fn is_running(&self) -> bool {
        let state = self.inner.read().await;
        matches!(
            state.current_run.as_ref().map(|r| &r.status),
            Some(ScraperStatus::Running)
        )
    }

    pub async fn get_current_run(&self) -> Option<ScrapingRun> {
        let state = self.inner.read().await;
        state.current_run.clone()
    }

    pub async fn set_csv(&self, content: String, info: CsvInfo) {
        let mut state = self.inner.write().await;
        state.csv_content = Some(content);
        state.csv_info = Some(info);
    }

    pub async fn get_csv(&self) -> Option<(String, CsvInfo)> {
        let state = self.inner.read().await;
        match (&state.csv_content, &state.csv_info) {
            (Some(content), Some(info)) => Some((content.to_string(), info.clone())),
            _ => None,
        }
    }

    pub async fn clear_csv(&self) {
        let mut state = self.inner.write().await;
        state.csv_content = None;
        state.csv_info = None;
    }

    pub async fn start_run(&self, run_id: i32, total_modules: usize) {
        let mut state = self.inner.write().await;
        state.current_run = Some(ScrapingRun {
            run_id,
            status: ScraperStatus::Running,
            total_modules,
            completed: 0,
            successful: 0,
            failed: 0,
            skipped: 0,
            started_at: chrono::Utc::now(),
            logs: Vec::new(),
        });

        let _ = self.event_tx.send(ScraperEvent::Started {
            run_id,
            total_modules,
        });
    }

    pub async fn update_progress(
        &self,
        completed: usize,
        successful: usize,
        failed: usize,
        skipped: usize,
    ) {
        let mut state = self.inner.write().await;
        if let Some(run) = &mut state.current_run {
            run.completed = completed;
            run.successful = successful;
            run.failed = failed;
            run.skipped = skipped;

            let _ = self.event_tx.send(ScraperEvent::Progress {
                completed,
                total: run.total_modules,
                successful,
                failed,
                skipped,
            });
        }
    }

    pub async fn add_log(&self, message: String, level: LogLevel) {
        let mut state = self.inner.write().await;
        if let Some(run) = &mut state.current_run {
            run.logs.push(message.clone());

            // Keep only the last MAX_LOG_ENTRIES
            if run.logs.len() > MAX_LOG_ENTRIES {
                run.logs.drain(0..run.logs.len() - MAX_LOG_ENTRIES);
            }
        }

        let _ = self.event_tx.send(ScraperEvent::Log { message, level });
    }

    pub async fn complete_run(&self, successful: usize, failed: usize, skipped: usize) {
        let mut state = self.inner.write().await;
        if let Some(run) = &mut state.current_run {
            run.status = ScraperStatus::Completed;
            run.successful = successful;
            run.failed = failed;
            run.skipped = skipped;
        }

        let _ = self.event_tx.send(ScraperEvent::Completed {
            successful,
            failed,
            skipped,
        });
    }

    pub async fn fail_run(&self, error: String) {
        let mut state = self.inner.write().await;
        if let Some(run) = &mut state.current_run {
            run.status = ScraperStatus::Failed;
        }

        let _ = self.event_tx.send(ScraperEvent::Failed { error });
    }

    pub fn subscribe(&self) -> broadcast::Receiver<ScraperEvent> {
        self.event_tx.subscribe()
    }
}

impl Default for ScraperState {
    fn default() -> Self {
        Self::new()
    }
}
