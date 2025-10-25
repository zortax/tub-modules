use anyhow::Result;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::{db_ops, mapper, module, search::ModuleRef};

/// Configuration for a scraping run
#[derive(Debug, Clone)]
pub struct ScraperConfig {
    pub retries: u32,
    pub num_workers: usize,
}

impl Default for ScraperConfig {
    fn default() -> Self {
        Self {
            retries: 3,
            num_workers: num_cpus::get(),
        }
    }
}

/// Progress information for a scraping run
#[derive(Debug, Clone)]
pub struct ScraperProgress {
    pub total: usize,
    pub completed: usize,
    pub successful: usize,
    pub failed: usize,
    pub skipped: usize,
}

impl ScraperProgress {
    pub fn new(total: usize) -> Self {
        Self {
            total,
            completed: 0,
            successful: 0,
            failed: 0,
            skipped: 0,
        }
    }
}

/// Events emitted during scraping
#[derive(Debug, Clone)]
pub enum ScraperEvent {
    Started { total_modules: usize },
    Progress {
        current: usize,
        total: usize,
        successful: usize,
        failed: usize,
        skipped: usize,
    },
    ModuleSuccess { number: i32, version: i32, title: String },
    ModuleSkipped { number: i32, version: i32, reason: String },
    ModuleFailed { number: i32, version: i32, error: String },
    Completed { successful: usize, failed: usize, skipped: usize },
}

/// Run the scraper with the given modules and configuration
///
/// This function takes ownership of the progress tracker and event callback,
/// allowing the caller to monitor progress in real-time.
pub async fn run_scraper<F>(
    pool: Arc<PgPool>,
    modules: Vec<ModuleRef>,
    scraping_run_id: i32,
    config: ScraperConfig,
    mut on_event: F,
) -> Result<ScraperProgress>
where
    F: FnMut(ScraperEvent) + Send + 'static,
{
    use futures::stream::{FuturesUnordered, StreamExt};

    let total = modules.len();
    let progress = Arc::new(RwLock::new(ScraperProgress::new(total)));

    on_event(ScraperEvent::Started { total_modules: total });

    // Create semaphore for worker pool
    let semaphore = Arc::new(tokio::sync::Semaphore::new(config.num_workers));

    // Process modules concurrently
    let mut tasks = FuturesUnordered::new();

    for module_ref in modules {
        let pool = Arc::clone(&pool);
        let progress = Arc::clone(&progress);
        let semaphore = Arc::clone(&semaphore);
        let retries = config.retries;

        let task = tokio::spawn(async move {
            let _permit = semaphore.acquire().await.unwrap();

            let result = process_module(&pool, &module_ref, scraping_run_id, retries).await;

            // Update progress
            let mut prog = progress.write().await;
            prog.completed += 1;

            let event = match result {
                Ok(true) => {
                    prog.successful += 1;
                    ScraperEvent::ModuleSuccess {
                        number: module_ref.number,
                        version: module_ref.version,
                        title: module_ref.title.clone(),
                    }
                }
                Ok(false) => {
                    prog.skipped += 1;
                    ScraperEvent::ModuleSkipped {
                        number: module_ref.number,
                        version: module_ref.version,
                        reason: "Authentication required".to_string(),
                    }
                }
                Err(e) => {
                    prog.failed += 1;
                    ScraperEvent::ModuleFailed {
                        number: module_ref.number,
                        version: module_ref.version,
                        error: e.to_string(),
                    }
                }
            };

            (event, prog.completed, prog.total, prog.successful, prog.failed, prog.skipped)
        });

        tasks.push(task);
    }

    // Collect events as tasks complete
    while let Some(result) = tasks.next().await {
        if let Ok((event, current, total, successful, failed, skipped)) = result {
            on_event(ScraperEvent::Progress {
                current,
                total,
                successful,
                failed,
                skipped,
            });
            on_event(event);
        }
    }

    // Get final progress
    let final_progress = {
        let p = progress.read().await;
        p.clone()
    };

    on_event(ScraperEvent::Completed {
        successful: final_progress.successful,
        failed: final_progress.failed,
        skipped: final_progress.skipped,
    });

    Ok(final_progress)
}

async fn process_module(
    pool: &PgPool,
    module_ref: &ModuleRef,
    scraping_run_id: i32,
    retries: u32,
) -> Result<bool> {
    // Fetch module details
    let scraped_module = match module::fetch_module_details(&module_ref.detail_url, retries).await? {
        Some(m) => m,
        None => return Ok(false), // Authentication required
    };

    // Map to database models
    let mapped_data = mapper::map_module_data(pool, scraped_module, scraping_run_id).await?;

    // Insert into database
    db_ops::insert_module_data(pool, mapped_data).await?;

    Ok(true)
}
