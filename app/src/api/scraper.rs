use leptos::prelude::*;
use serde::{Deserialize, Serialize};

// Import shared types
use crate::scraper_types::ScrapingRun;

#[cfg(feature = "ssr")]
use actix_web::{web, HttpRequest, HttpResponse};

#[cfg(feature = "ssr")]
use crate::scraper_state::{CsvInfo, LogLevel, ScraperState};

#[cfg(feature = "ssr")]
use leptos_actix::extract;

#[cfg(feature = "ssr")]
use moses_scraper::{parse_csv_content, run_scraper, validate_csv_content, ScraperConfig};

#[cfg(feature = "ssr")]
use std::sync::Arc;

/// Request to upload and validate CSV
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadCsvRequest {
    pub content: String,
    pub filename: String,
}

/// Response from CSV upload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadCsvResponse {
    pub success: bool,
    pub message: String,
    pub info: Option<CsvInfoDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CsvInfoDto {
    pub total_rows: usize,
    pub valid_modules: usize,
    pub invalid_rows: usize,
    pub file_name: String,
}

/// Response from starting scraper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartScraperResponse {
    pub success: bool,
    pub message: String,
    pub run_id: Option<i32>,
}

/// Upload and validate CSV file
#[server(UploadCsv)]
pub async fn upload_csv(
    content: String,
    filename: String,
    auth_key: String,
) -> Result<UploadCsvResponse, ServerFnError> {
    // Check auth
    if !check_auth(&auth_key).await? {
        return Ok(UploadCsvResponse {
            success: false,
            message: "Invalid authentication key".to_string(),
            info: None,
        });
    }

    // Validate CSV
    match validate_csv_content(&content) {
        Ok(validation_result) => {
            let info = CsvInfo {
                total_rows: validation_result.total_rows,
                valid_modules: validation_result.valid_modules,
                invalid_rows: validation_result.invalid_rows,
                file_name: filename.clone(),
            };

            // Store in scraper state
            let state = extract::<web::Data<ScraperState>>().await?;
            state.set_csv(content, info.clone()).await;

            Ok(UploadCsvResponse {
                success: true,
                message: "CSV validated successfully".to_string(),
                info: Some(CsvInfoDto {
                    total_rows: info.total_rows,
                    valid_modules: info.valid_modules,
                    invalid_rows: info.invalid_rows,
                    file_name: info.file_name,
                }),
            })
        }
        Err(e) => Ok(UploadCsvResponse {
            success: false,
            message: format!("CSV validation failed: {}", e),
            info: None,
        }),
    }
}

/// Start scraping run
#[server(StartScraper)]
pub async fn start_scraper(
    auth_key: String,
    workers: u32,
    url_pattern: Option<String>,
) -> Result<StartScraperResponse, ServerFnError> {
    // Check auth
    if !check_auth(&auth_key).await? {
        return Ok(StartScraperResponse {
            success: false,
            message: "Invalid authentication key".to_string(),
            run_id: None,
        });
    }

    let state = extract::<web::Data<ScraperState>>().await?;

    // Check if already running
    if state.is_running().await {
        return Ok(StartScraperResponse {
            success: false,
            message: "A scraping run is already in progress".to_string(),
            run_id: None,
        });
    }

    // Get CSV content
    let (csv_content, _csv_info) = match state.get_csv().await {
        Some(data) => data,
        None => {
            return Ok(StartScraperResponse {
                success: false,
                message: "No CSV file uploaded".to_string(),
                run_id: None,
            });
        }
    };

    // Parse CSV
    let modules = match parse_csv_content(&csv_content, None, url_pattern.as_deref()) {
        Ok(m) => m,
        Err(e) => {
            return Ok(StartScraperResponse {
                success: false,
                message: format!("Failed to parse CSV: {}", e),
                run_id: None,
            });
        }
    };

    // Get database pool
    let pool = extract::<web::Data<db::PgPool>>().await?;
    let pool = Arc::new((**pool).clone());

    // Create scraping run in database
    let scraping_run_id = sqlx::query!(
        r#"
        INSERT INTO scraping_run (status, total_modules)
        VALUES ('in_progress', $1)
        RETURNING id
        "#,
        modules.len() as i32
    )
    .fetch_one(&*pool)
    .await?
    .id;

    tracing::info!("Created scraping run with ID: {}", scraping_run_id);

    // Initialize state
    state.start_run(scraping_run_id, modules.len()).await;

    // Spawn scraping task
    let state_clone = state.get_ref().clone();
    let pool_clone = Arc::clone(&pool);

    tokio::spawn(async move {
        let config = ScraperConfig {
            retries: 3,
            num_workers: if workers > 0 { workers as usize } else { 1 },
        };

        state_clone
            .add_log(
                format!("Starting scrape of {} modules...", modules.len()),
                LogLevel::Info,
            )
            .await;

        let result = run_scraper(pool_clone.clone(), modules, scraping_run_id, config, {
            let state = state_clone.clone();
            move |event| {
                let state = state.clone();
                tokio::spawn(async move {
                    match event {
                        moses_scraper::ScraperEvent::Started { .. } => {
                            // Already logged
                        }
                        moses_scraper::ScraperEvent::Progress {
                            current,
                            successful,
                            failed,
                            skipped,
                            ..
                        } => {
                            state
                                .update_progress(current, successful, failed, skipped)
                                .await;
                        }
                        moses_scraper::ScraperEvent::ModuleSuccess {
                            number,
                            version,
                            title,
                        } => {
                            state
                                .add_log(
                                    format!("✓ {} v{}: {}", number, version, title),
                                    LogLevel::Success,
                                )
                                .await;
                        }
                        moses_scraper::ScraperEvent::ModuleSkipped {
                            number,
                            version,
                            reason,
                        } => {
                            state
                                .add_log(
                                    format!("⊘ {} v{}: {}", number, version, reason),
                                    LogLevel::Warning,
                                )
                                .await;
                        }
                        moses_scraper::ScraperEvent::ModuleFailed {
                            number,
                            version,
                            error,
                        } => {
                            state
                                .add_log(
                                    format!("✗ {} v{}: {}", number, version, error),
                                    LogLevel::Error,
                                )
                                .await;
                        }
                        moses_scraper::ScraperEvent::Completed {
                            successful,
                            failed,
                            skipped,
                        } => {
                            state
                                .add_log(
                                    format!(
                                        "Scraping completed: {} successful, {} failed, {} skipped",
                                        successful, failed, skipped
                                    ),
                                    LogLevel::Info,
                                )
                                .await;
                        }
                    }
                });
            }
        })
        .await;

        match result {
            Ok(progress) => {
                // Update final state
                state_clone
                    .complete_run(progress.successful, progress.failed, progress.skipped)
                    .await;

                // Update database
                let _ = sqlx::query!(
                    r#"
                    UPDATE scraping_run
                    SET completed_at = NOW(),
                        status = 'completed',
                        successful_modules = $1,
                        failed_modules = $2,
                        skipped_modules = $3
                    WHERE id = $4
                    "#,
                    progress.successful as i32,
                    progress.failed as i32,
                    progress.skipped as i32,
                    scraping_run_id
                )
                .execute(&*pool_clone)
                .await;

                tracing::info!("Scraping run {} completed successfully", scraping_run_id);
            }
            Err(e) => {
                let error_msg = format!("Scraping failed: {}", e);
                state_clone.fail_run(error_msg.clone()).await;
                state_clone.add_log(error_msg, LogLevel::Error).await;

                // Update database
                let _ = sqlx::query!(
                    r#"
                    UPDATE scraping_run
                    SET completed_at = NOW(),
                        status = 'failed'
                    WHERE id = $1
                    "#,
                    scraping_run_id
                )
                .execute(&*pool_clone)
                .await;

                tracing::error!("Scraping run {} failed", scraping_run_id);
            }
        }

        // Clear CSV after completion
        state_clone.clear_csv().await;
    });

    Ok(StartScraperResponse {
        success: true,
        message: "Scraping started successfully".to_string(),
        run_id: Some(scraping_run_id),
    })
}

/// Get current scraper status
#[server(GetScraperStatus)]
pub async fn get_scraper_status(auth_key: String) -> Result<Option<ScrapingRun>, ServerFnError> {
    // Check auth
    if !check_auth(&auth_key).await? {
        return Err(ServerFnError::new("Invalid authentication key"));
    }

    let state = extract::<web::Data<ScraperState>>().await?;
    Ok(state.get_current_run().await)
}

#[cfg(feature = "ssr")]
async fn check_auth(provided_key: &str) -> Result<bool, ServerFnError> {
    let expected_key = std::env::var("SCRAPER_AUTH_KEY")
        .map_err(|_| ServerFnError::new("SCRAPER_AUTH_KEY not configured"))?;

    Ok(provided_key == expected_key)
}

/// SSE endpoint for scraper events
#[cfg(feature = "ssr")]
pub async fn scraper_events_sse(
    req: HttpRequest,
    state: web::Data<ScraperState>,
) -> HttpResponse {
    use actix_web::rt::time::interval;
    use futures::stream::Stream;
    use std::pin::Pin;
    use std::time::Duration;

    // Check auth via query parameter
    let auth_key = req
        .query_string()
        .split('&')
        .find(|s| s.starts_with("auth_key="))
        .and_then(|s| s.strip_prefix("auth_key="))
        .unwrap_or("");

    let expected_key = match std::env::var("SCRAPER_AUTH_KEY") {
        Ok(k) => k,
        Err(_) => {
            return HttpResponse::Unauthorized().body("SCRAPER_AUTH_KEY not configured");
        }
    };

    if auth_key != expected_key {
        return HttpResponse::Unauthorized().body("Invalid authentication key");
    }

    let mut rx = state.subscribe();
    let state_clone = state.clone();

    let stream: Pin<Box<dyn Stream<Item = Result<actix_web::web::Bytes, actix_web::Error>>>> =
        Box::pin(async_stream::stream! {
            // Send initial state as proper events
            if let Some(run) = state_clone.get_current_run().await {
                // Send Started event first
                let started_event = crate::scraper_types::ScraperEvent::Started {
                    run_id: run.run_id,
                    total_modules: run.total_modules,
                };
                if let Ok(event_json) = serde_json::to_string(&started_event) {
                    yield Ok(actix_web::web::Bytes::from(format!("data: {}\n\n", event_json)));
                }

                // Send Progress event with current state
                let progress_event = crate::scraper_types::ScraperEvent::Progress {
                    completed: run.completed,
                    total: run.total_modules,
                    successful: run.successful,
                    failed: run.failed,
                    skipped: run.skipped,
                };
                if let Ok(event_json) = serde_json::to_string(&progress_event) {
                    yield Ok(actix_web::web::Bytes::from(format!("data: {}\n\n", event_json)));
                }

                // Send existing logs
                for log in run.logs {
                    let log_event = crate::scraper_types::ScraperEvent::Log {
                        message: log,
                        level: crate::scraper_types::LogLevel::Info,
                    };
                    if let Ok(event_json) = serde_json::to_string(&log_event) {
                        yield Ok(actix_web::web::Bytes::from(format!("data: {}\n\n", event_json)));
                    }
                }
            }

            // Send keepalive every 30 seconds
            let mut keepalive = interval(Duration::from_secs(30));

            loop {
                tokio::select! {
                    result = rx.recv() => {
                        match result {
                            Ok(event) => {
                                let event_json = serde_json::to_string(&event).unwrap_or_default();
                                yield Ok(actix_web::web::Bytes::from(format!("data: {}\n\n", event_json)));
                            }
                            Err(_) => {
                                // Channel closed or lagged
                                break;
                            }
                        }
                    }
                    _ = keepalive.tick() => {
                        yield Ok(actix_web::web::Bytes::from(": keepalive\n\n"));
                    }
                }
            }
        });

    HttpResponse::Ok()
        .content_type("text/event-stream")
        .streaming(stream)
}
