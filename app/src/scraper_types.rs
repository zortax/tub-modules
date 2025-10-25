use serde::{Deserialize, Serialize};

/// Shared types for scraper state (available on both client and server)

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScrapingRun {
    pub run_id: i32,
    pub status: ScraperStatus,
    pub total_modules: usize,
    pub completed: usize,
    pub successful: usize,
    pub failed: usize,
    pub skipped: usize,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub logs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ScraperStatus {
    Idle,
    Running,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ScraperEvent {
    Started {
        run_id: i32,
        total_modules: usize,
    },
    Progress {
        completed: usize,
        total: usize,
        successful: usize,
        failed: usize,
        skipped: usize,
    },
    Log {
        message: String,
        level: LogLevel,
    },
    Completed {
        successful: usize,
        failed: usize,
        skipped: usize,
    },
    Failed {
        error: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Info,
    Success,
    Warning,
    Error,
}
