use leptos::prelude::*;

#[component]
pub fn ScraperProgress(
    total: Signal<usize>,
    completed: Signal<usize>,
    successful: Signal<usize>,
    failed: Signal<usize>,
    skipped: Signal<usize>,
) -> impl IntoView {
    let progress_percent = move || {
        let t = total.get();
        if t == 0 {
            0.0
        } else {
            (completed.get() as f64 / t as f64) * 100.0
        }
    };

    view! {
        <div class="card bg-base-100 shadow-xl">
            <div class="card-body">
                <h2 class="card-title">
                    <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z" />
                    </svg>
                    "Scraping Progress"
                </h2>

                <div class="space-y-4">
                    // Progress bar
                    <div>
                        <div class="flex justify-between text-sm mb-2">
                            <span class="font-semibold">{move || completed.get()} " / " {move || total.get()}</span>
                            <span class="text-base-content/60">{move || format!("{:.1}%", progress_percent())}</span>
                        </div>
                        <progress
                            class="progress progress-primary w-full h-4"
                            value=move || completed.get()
                            max=move || total.get()
                        />
                    </div>

                    // Stats
                    <div class="grid grid-cols-3 gap-4">
                        <div class="stat bg-success/10 rounded-lg p-4">
                            <div class="stat-title text-sm">Successful</div>
                            <div class="stat-value text-2xl text-success">{move || successful.get()}</div>
                        </div>

                        <div class="stat bg-warning/10 rounded-lg p-4">
                            <div class="stat-title text-sm">Skipped</div>
                            <div class="stat-value text-2xl text-warning">{move || skipped.get()}</div>
                        </div>

                        <div class="stat bg-error/10 rounded-lg p-4">
                            <div class="stat-title text-sm">Failed</div>
                            <div class="stat-value text-2xl text-error">{move || failed.get()}</div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}
