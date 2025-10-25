use leptos::prelude::*;
use std::collections::VecDeque;

#[component]
pub fn ScraperLogs(logs: Signal<VecDeque<LogEntry>>) -> impl IntoView {
    let logs_container_ref = NodeRef::<leptos::html::Div>::new();

    // Auto-scroll to bottom when new logs arrive
    Effect::new(move || {
        let _ = logs.get(); // Track changes
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(container) = logs_container_ref.get() {
                use wasm_bindgen::JsCast;
                let element = container.unchecked_into::<web_sys::HtmlElement>();
                let _ = element.set_scroll_top(element.scroll_height());
            }
        }
    });

    view! {
        <div class="card bg-base-100 shadow-xl h-full flex flex-col">
            <div class="card-body flex-1 flex flex-col min-h-0">
                <h2 class="card-title flex-none">
                    <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
                    </svg>
                    "Scraping Logs"
                </h2>

                <div
                    node_ref=logs_container_ref
                    class="flex-1 overflow-y-auto bg-base-200 rounded-lg p-4 font-mono text-sm space-y-1 min-h-0"
                >
                    {move || logs.get().iter().map(|log| {
                        let level_class = match log.level.as_str() {
                            "success" => "text-success",
                            "warning" => "text-warning",
                            "error" => "text-error",
                            _ => "text-base-content/80",
                        };
                        let timestamp = log.timestamp;
                        let message = log.message.clone();

                        view! {
                            <div class=format!("flex items-start gap-2 {}", level_class)>
                                <span class="text-base-content/40 shrink-0 text-xs">
                                    {format_timestamp(&timestamp)}
                                </span>
                                <span class="flex-1 break-all">{message}</span>
                            </div>
                        }
                    }).collect::<Vec<_>>()}

                    {move || {
                        if logs.get().is_empty() {
                            view! {
                                <div class="text-base-content/40 text-center py-8">
                                    "No logs yet. Start a scraping run to see logs here."
                                </div>
                            }.into_any()
                        } else {
                            view! { <></> }.into_any()
                        }
                    }}
                </div>
            </div>
        </div>
    }
}

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub message: String,
    pub level: String,
    pub timestamp: f64,
}

fn format_timestamp(timestamp: &f64) -> String {
    #[cfg(target_arch = "wasm32")]
    {
        use web_sys::js_sys;

        let date = js_sys::Date::new(&(*timestamp).into());
        let hours = date.get_hours();
        let minutes = date.get_minutes();
        let seconds = date.get_seconds();

        format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = timestamp;
        "00:00:00".to_string()
    }
}
