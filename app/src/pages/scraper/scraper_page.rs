use leptos::prelude::*;
use leptos::task::spawn_local;
use std::collections::VecDeque;

use crate::api::{get_scraper_status, start_scraper, CsvInfoDto};
use crate::components::PageLayout;

use super::csv_upload::CsvUpload;
use super::scraper_logs::{LogEntry, ScraperLogs};
use super::scraper_progress::ScraperProgress;

#[cfg(target_arch = "wasm32")]
const MAX_VISIBLE_LOGS: usize = 100;

#[component]
pub fn ScraperPage() -> impl IntoView {
    let (auth_key, set_auth_key) = signal(String::new());
    let (is_authenticated, set_is_authenticated) = signal(false);
    let (csv_info, set_csv_info) = signal(None::<CsvInfoDto>);
    let (is_scraping, set_is_scraping) = signal(false);
    let (error, set_error) = signal(None::<String>);

    // Configuration options
    let (workers, set_workers) = signal(1_u32);
    let (url_pattern, set_url_pattern) = signal(String::from(
        "https://moseskonto.tu-berlin.de/moses/modultransfersystem/bolognamodule/beschreibung/anzeigen.html?nummer={number}&version={version}&sprache=1"
    ));

    // Scraping progress
    let (total, set_total) = signal(0_usize);
    let (completed, set_completed) = signal(0_usize);
    let (successful, set_successful) = signal(0_usize);
    let (failed, set_failed) = signal(0_usize);
    let (skipped, set_skipped) = signal(0_usize);

    // Logs
    let (logs, set_logs) = signal(VecDeque::<LogEntry>::new());

    let on_auth_submit = move |_| {
        let key = auth_key.get_untracked();
        if !key.is_empty() {
            set_is_authenticated.set(true);

            // Check for existing scraping run
            spawn_local(async move {
                if let Ok(Some(run)) = get_scraper_status(key.clone()).await {
                    set_total.set(run.total_modules);
                    set_completed.set(run.completed);
                    set_successful.set(run.successful);
                    set_failed.set(run.failed);
                    set_skipped.set(run.skipped);

                    if run.status == crate::scraper_types::ScraperStatus::Running {
                        set_is_scraping.set(true);
                        start_sse_connection(
                            key.clone(),
                            set_total,
                            set_completed,
                            set_successful,
                            set_failed,
                            set_skipped,
                            set_logs,
                            set_is_scraping,
                        );
                    }

                    // Load existing logs
                    let mut log_queue = VecDeque::new();
                    for log in run.logs {
                        log_queue.push_back(LogEntry {
                            message: log,
                            level: "info".to_string(),
                            timestamp: js_sys::Date::now(),
                        });
                    }
                    set_logs.set(log_queue);
                }
            });
        }
    };

    let on_csv_uploaded = move |info: CsvInfoDto| {
        set_csv_info.set(Some(info));
        set_error.set(None);
    };

    let on_start_scraping = move |_| {
        let key = auth_key.get_untracked();
        let num_workers = workers.get_untracked();
        let pattern = url_pattern.get_untracked();
        let pattern_opt = if pattern.is_empty() { None } else { Some(pattern) };

        spawn_local(async move {
            set_error.set(None);

            match start_scraper(key.clone(), num_workers, pattern_opt).await {
                Ok(response) => {
                    if response.success {
                        set_is_scraping.set(true);
                        set_csv_info.set(None); // Clear CSV info after starting

                        // Start SSE connection
                        start_sse_connection(
                            key,
                            set_total,
                            set_completed,
                            set_successful,
                            set_failed,
                            set_skipped,
                            set_logs,
                            set_is_scraping,
                        );
                    } else {
                        set_error.set(Some(response.message));
                    }
                }
                Err(e) => {
                    set_error.set(Some(format!("Failed to start scraper: {}", e)));
                }
            }
        });
    };

    view! {
        <PageLayout>
            <div class="container mx-auto px-4 py-8">
                <h1 class="text-4xl font-bold mb-8">Module Scraper</h1>

                {move || {
                    if !is_authenticated.get() {
                        // Authentication form
                        view! {
                            <div class="max-w-md mx-auto">
                                <div class="card bg-base-100 shadow-xl">
                                    <div class="card-body">
                                        <h2 class="card-title">Authentication Required</h2>
                                        <p class="text-sm text-base-content/60 mb-4">
                                            "Enter the scraper authentication key to access this page."
                                        </p>

                                        <form on:submit=on_auth_submit class="space-y-4">
                                            <div class="form-control">
                                                <label class="label">
                                                    <span class="label-text">Auth Key</span>
                                                </label>
                                                <input
                                                    type="password"
                                                    placeholder="Enter auth key"
                                                    class="input input-bordered w-full"
                                                    prop:value=move || auth_key.get()
                                                    on:input=move |ev| set_auth_key.set(event_target_value(&ev))
                                                />
                                            </div>

                                            <button
                                                type="submit"
                                                class="btn btn-primary btn-soft w-full"
                                                disabled=move || auth_key.get().is_empty()
                                            >
                                                "Access Scraper"
                                            </button>
                                        </form>
                                    </div>
                                </div>
                            </div>
                        }.into_any()
                    } else if is_scraping.get() {
                        // Scraping view
                        view! {
                            <div class="grid grid-cols-1 gap-6">
                                <ScraperProgress
                                    total=total.into()
                                    completed=completed.into()
                                    successful=successful.into()
                                    failed=failed.into()
                                    skipped=skipped.into()
                                />

                                <div class="h-[600px]">
                                    <ScraperLogs logs=logs.into() />
                                </div>
                            </div>
                        }.into_any()
                    } else {
                        // Upload and start view
                        view! {
                            <div class="space-y-6">
                                <CsvUpload
                                    auth_key=auth_key.into()
                                    on_uploaded=on_csv_uploaded
                                />

                                {move || csv_info.get().map(|info| view! {
                                    <div class="card bg-base-100 shadow-xl">
                                        <div class="card-body">
                                            <h2 class="card-title">CSV Information</h2>

                                            <div class="stats stats-vertical lg:stats-horizontal shadow">
                                                <div class="stat">
                                                    <div class="stat-title">Total Rows</div>
                                                    <div class="stat-value text-primary">{info.total_rows}</div>
                                                </div>

                                                <div class="stat">
                                                    <div class="stat-title">Valid Modules</div>
                                                    <div class="stat-value text-success">{info.valid_modules}</div>
                                                </div>

                                                <div class="stat">
                                                    <div class="stat-title">Invalid Rows</div>
                                                    <div class="stat-value text-error">{info.invalid_rows}</div>
                                                </div>
                                            </div>

                                            <div class="divider"></div>

                                            <h3 class="font-semibold text-lg">Scraper Configuration</h3>

                                            <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                                                <div class="form-control">
                                                    <label class="label">
                                                        <span class="label-text">Number of Workers</span>
                                                    </label>
                                                    <input
                                                        type="number"
                                                        min="1"
                                                        max="100"
                                                        class="input input-bordered"
                                                        prop:value=move || workers.get()
                                                        on:input=move |ev| {
                                                            if let Ok(val) = event_target_value(&ev).parse::<u32>() {
                                                                set_workers.set(val.max(1));
                                                            }
                                                        }
                                                    />
                                                    <label class="label">
                                                        <span class="label-text-alt">Concurrent modules to scrape (default: 1)</span>
                                                    </label>
                                                </div>

                                                <div class="form-control">
                                                    <label class="label">
                                                        <span class="label-text">URL Pattern (optional)</span>
                                                    </label>
                                                    <input
                                                        type="text"
                                                        placeholder="{number} and {version} placeholders"
                                                        class="input input-bordered"
                                                        prop:value=move || url_pattern.get()
                                                        on:input=move |ev| set_url_pattern.set(event_target_value(&ev))
                                                    />
                                                    <label class="label">
                                                        <span class="label-text-alt">Leave empty for default MOSES URL</span>
                                                    </label>
                                                </div>
                                            </div>

                                            <div class="card-actions justify-end mt-4">
                                                <button
                                                    class="btn btn-primary btn-soft"
                                                    on:click=on_start_scraping
                                                >
                                                    <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5 mr-2" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z" />
                                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                                                    </svg>
                                                    "Start Scraping"
                                                </button>
                                            </div>
                                        </div>
                                    </div>
                                })}

                                {move || error.get().map(|err| view! {
                                    <div class="alert alert-error">
                                        <svg xmlns="http://www.w3.org/2000/svg" class="stroke-current shrink-0 h-6 w-6" fill="none" viewBox="0 0 24 24">
                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2m7-2a9 9 0 11-18 0 9 9 0 0118 0z" />
                                        </svg>
                                        <span>{err}</span>
                                    </div>
                                })}
                            </div>
                        }.into_any()
                    }
                }}
            </div>
        </PageLayout>
    }
}

#[cfg(target_arch = "wasm32")]
fn start_sse_connection(
    auth_key: String,
    set_total: WriteSignal<usize>,
    set_completed: WriteSignal<usize>,
    set_successful: WriteSignal<usize>,
    set_failed: WriteSignal<usize>,
    set_skipped: WriteSignal<usize>,
    set_logs: WriteSignal<VecDeque<LogEntry>>,
    set_is_scraping: WriteSignal<bool>,
) {
    use wasm_bindgen::JsCast;
    use web_sys::{EventSource, MessageEvent};

    let url = format!("/api/scraper/events?auth_key={}", auth_key);

    if let Ok(event_source) = EventSource::new(&url) {
        let es_clone = event_source.clone();

        let onmessage = wasm_bindgen::closure::Closure::wrap(Box::new(move |e: MessageEvent| {
            if let Some(data) = e.data().as_string() {
                if let Ok(event) = serde_json::from_str::<crate::scraper_types::ScraperEvent>(&data) {
                    match event {
                        crate::scraper_types::ScraperEvent::Started { total_modules, .. } => {
                            set_total.set(total_modules);
                            set_completed.set(0);
                            set_successful.set(0);
                            set_failed.set(0);
                            set_skipped.set(0);
                        }
                        crate::scraper_types::ScraperEvent::Progress {
                            completed,
                            total,
                            successful,
                            failed,
                            skipped,
                        } => {
                            set_total.set(total);
                            set_completed.set(completed);
                            set_successful.set(successful);
                            set_failed.set(failed);
                            set_skipped.set(skipped);
                        }
                        crate::scraper_types::ScraperEvent::Log { message, level } => {
                            set_logs.update(|logs| {
                                logs.push_back(LogEntry {
                                    message,
                                    level: format!("{:?}", level).to_lowercase(),
                                    timestamp: js_sys::Date::now(),
                                });

                                // Keep only last MAX_VISIBLE_LOGS
                                while logs.len() > MAX_VISIBLE_LOGS {
                                    logs.pop_front();
                                }
                            });
                        }
                        crate::scraper_types::ScraperEvent::Completed { .. } => {
                            set_is_scraping.set(false);
                            es_clone.close();
                        }
                        crate::scraper_types::ScraperEvent::Failed { .. } => {
                            set_is_scraping.set(false);
                            es_clone.close();
                        }
                    }
                }
            }
        }) as Box<dyn FnMut(_)>);

        event_source.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
        onmessage.forget();
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn start_sse_connection(
    _auth_key: String,
    _set_total: WriteSignal<usize>,
    _set_completed: WriteSignal<usize>,
    _set_successful: WriteSignal<usize>,
    _set_failed: WriteSignal<usize>,
    _set_skipped: WriteSignal<usize>,
    _set_logs: WriteSignal<VecDeque<LogEntry>>,
    _set_is_scraping: WriteSignal<bool>,
) {
    // SSR: no-op
}

// Import js_sys for WASM
#[cfg(target_arch = "wasm32")]
use web_sys::js_sys;

// Mock for SSR
#[cfg(not(target_arch = "wasm32"))]
mod js_sys {
    pub struct Date;
    impl Date {
        pub fn now() -> f64 {
            0.0
        }
    }
}
