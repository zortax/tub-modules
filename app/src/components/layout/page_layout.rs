use leptos::prelude::*;
use crate::api::get_latest_scraping_run;
use chrono::{DateTime, Utc, Local};

#[component]
pub fn PageLayout(children: Children) -> impl IntoView {
    // Fetch the latest scraping run timestamp
    let last_updated = Resource::new(|| (), |_| async {
        get_latest_scraping_run().await.ok().flatten()
    });

    // Format the timestamp for display
    let format_timestamp = move |dt: DateTime<Utc>| -> String {
        let now = chrono::Utc::now();
        let duration = now.signed_duration_since(dt);

        if duration.num_minutes() < 60 {
            format!("{} minute{} ago", duration.num_minutes(), if duration.num_minutes() == 1 { "" } else { "s" })
        } else if duration.num_hours() < 24 {
            format!("{} hour{} ago", duration.num_hours(), if duration.num_hours() == 1 { "" } else { "s" })
        } else if duration.num_days() < 7 {
            format!("{} day{} ago", duration.num_days(), if duration.num_days() == 1 { "" } else { "s" })
        } else {
            dt.format("%b %d, %Y at %H:%M UTC").to_string()
        }
    };

    view! {
        <div class="min-h-screen bg-base-300">
            // Fixed header
            <header class="sticky top-0 z-50 bg-base-100 shadow-lg border-b border-base-300">
                <div class="w-full px-4 py-4">
                    // Use same grid layout as main content to align with filter panel
                    // Max width accounts for: 380px filter + 24px gap + 896px (max-w-4xl) = ~1300px
                    <div class="grid grid-cols-1 lg:grid-cols-[380px_1fr] gap-6 max-w-[1300px] mx-auto">
                        // Header content aligned with filter panel on desktop, full width on mobile
                        <div class="flex items-center justify-between">
                            <div class="flex items-center gap-3 flex-1 min-w-0">
                                <svg xmlns="http://www.w3.org/2000/svg" class="h-8 w-8 text-primary shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6.253v13m0-13C10.832 5.477 9.246 5 7.5 5S4.168 5.477 3 6.253v13C4.168 18.477 5.754 18 7.5 18s3.332.477 4.5 1.253m0-13C13.168 5.477 14.754 5 16.5 5c1.747 0 3.332.477 4.5 1.253v13C19.832 18.477 18.247 18 16.5 18c-1.746 0-3.332.477-4.5 1.253" />
                                </svg>
                                <div class="min-w-0 flex-1">
                                    <h1 class="text-2xl font-bold leading-tight truncate">
                                        "TU Berlin Module Search"
                                    </h1>
                                    <p class="text-xs text-base-content/60 hidden sm:block">
                                        "Search and explore module descriptions"
                                    </p>
                                </div>
                            </div>
                            <a
                                href="https://github.com/zortax/tub-modules"
                                target="_blank"
                                rel="noopener noreferrer"
                                class="btn btn-ghost btn-sm gap-2 shrink-0 lg:hidden"
                            >
                                <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" fill="currentColor" viewBox="0 0 24 24">
                                    <path d="M12 0c-6.626 0-12 5.373-12 12 0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23.957-.266 1.983-.399 3.003-.404 1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576 4.765-1.589 8.199-6.086 8.199-11.386 0-6.627-5.373-12-12-12z"/>
                                </svg>
                                <span class="hidden sm:inline">
                                    "GitHub"
                                </span>
                            </a>
                        </div>

                        // Badge and GitHub link on desktop, aligned with search results
                        <div class="hidden lg:flex lg:items-center lg:justify-end lg:max-w-4xl lg:gap-2">
                            <Suspense fallback=move || view! { <span class="loading loading-spinner loading-xs"></span> }>
                                {move || last_updated.get().map(|timestamp_opt| {
                                    timestamp_opt.map(|timestamp| {
                                        let formatted = format_timestamp(timestamp);
                                        let local_time = timestamp.with_timezone(&Local);
                                        let full_time = format!("Last updated: {}", local_time.format("%B %d, %Y at %H:%M %Z"));
                                        view! {
                                            <span class="badge badge-sm badge-ghost tooltip tooltip-bottom whitespace-nowrap" data-tip=full_time.clone()>
                                                <svg xmlns="http://www.w3.org/2000/svg" class="h-3 w-3 mr-1" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
                                                </svg>
                                                {formatted}
                                            </span>
                                        }
                                    })
                                })}
                            </Suspense>
                            <a
                                href="https://github.com/zortax/tub-modules"
                                target="_blank"
                                rel="noopener noreferrer"
                                class="btn btn-ghost btn-sm gap-2"
                            >
                                <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" fill="currentColor" viewBox="0 0 24 24">
                                    <path d="M12 0c-6.626 0-12 5.373-12 12 0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23.957-.266 1.983-.399 3.003-.404 1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576 4.765-1.589 8.199-6.086 8.199-11.386 0-6.627-5.373-12-12-12z"/>
                                </svg>
                                "GitHub"
                            </a>
                        </div>
                    </div>
                </div>
            </header>

            // Main content - centered with same max-width as header
            <div class="w-full px-4 pt-6">
                <div class="max-w-[1300px] mx-auto">
                    {children()}
                </div>
            </div>
        </div>
    }
}
