use crate::components::filters::{
    ComponentLanguageFilter, ComponentTypeFilter, CreditFilter, ExamFilter, ProgramFilter,
    SemesterFilter,
};
use crate::models::{FilterOptions, SearchFilters};
use leptos::prelude::*;

#[component]
pub fn FilterPanel(
    filters: RwSignal<SearchFilters>,
    filter_options: Signal<Option<Result<FilterOptions, ServerFnError>>>,
) -> impl IntoView {
    let filter_options_ok = Memo::new(move |_| filter_options.get().and_then(|r| r.ok()));

    // Local search text state for debouncing
    let search_text = RwSignal::new(String::new());
    let debounce_timer = RwSignal::new(0_i32);

    view! {
        <div class="card bg-base-100 shadow-xl overflow-hidden max-h-[calc(100vh-7rem)]">
            <div class="card-body gap-4 overflow-y-auto pb-6">
                <div class="flex items-center justify-between">
                    <h2 class="card-title text-2xl flex items-center gap-2">
                        <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 4a1 1 0 011-1h16a1 1 0 011 1v2.586a1 1 0 01-.293.707l-6.414 6.414a1 1 0 00-.293.707V17l-4 4v-6.586a1 1 0 00-.293-.707L3.293 7.293A1 1 0 013 6.586V4z" />
                        </svg>
                        "Filters"
                    </h2>
                    <button
                        class="btn btn-ghost btn-sm btn-circle"
                        on:click=move |_| {
                            filters.set(SearchFilters::default());
                            search_text.set(String::new());
                        }
                        title="Clear all filters"
                    >
                        <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                        </svg>
                    </button>
                </div>

                <div class="divider my-0"></div>

                // Starred only checkbox
                <div class="form-control">
                    <label class="label cursor-pointer justify-start gap-3 py-1">
                        <input
                            type="checkbox"
                            class="checkbox checkbox-ghost"
                            prop:checked=move || filters.get().starred_only
                            on:change=move |ev| {
                                let checked = event_target_checked(&ev);
                                filters.update(|f| f.starred_only = checked);
                            }
                        />
                        <span class="label-text flex items-center gap-2">
                            <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 20 20" fill="currentColor">
                                <path d="M9.049 2.927c.3-.921 1.603-.921 1.902 0l1.07 3.292a1 1 0 00.95.69h3.462c.969 0 1.371 1.24.588 1.81l-2.8 2.034a1 1 0 00-.364 1.118l1.07 3.292c.3.921-.755 1.688-1.54 1.118l-2.8-2.034a1 1 0 00-1.175 0l-2.8 2.034c-.784.57-1.838-.197-1.539-1.118l1.07-3.292a1 1 0 00-.364-1.118L2.98 8.72c-.783-.57-.38-1.81.588-1.81h3.461a1 1 0 00.951-.69l1.07-3.292z" />
                            </svg>
                            "Show only starred"
                        </span>
                    </label>
                </div>

                <div class="divider my-0"></div>

                // Search input
                <div class="form-control">
                    <label class="label pb-1">
                        <span class="label-text font-semibold text-base flex items-center gap-2">
                            <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
                            </svg>
                            "Search"
                        </span>
                    </label>
                    <input
                        type="text"
                        placeholder="Search modules..."
                        class="input input-bordered w-full"
                        prop:value=move || search_text.get()
                        on:input=move |ev| {
                            let value = event_target_value(&ev);
                            search_text.set(value.clone());

                            // Increment timer to cancel previous timeout
                            let timer_id = debounce_timer.get() + 1;
                            debounce_timer.set(timer_id);

                            // Set new timeout
                            let _handle = gloo_timers::callback::Timeout::new(300, move || {
                                // Only update if timer hasn't changed (no newer input)
                                if debounce_timer.get_untracked() == timer_id {
                                    let mut new_filters = filters.get_untracked();
                                    new_filters.search_query = if value.is_empty() {
                                        None
                                    } else {
                                        Some(value)
                                    };
                                    filters.set(new_filters);
                                }
                            });
                            // Timer will auto-fire and cleanup
                            std::mem::forget(_handle);
                        }
                    />
                </div>

                <div class="divider my-0"></div>

                // Study program filter
                <ProgramFilter
                    filters=filters
                    filter_options=filter_options_ok.into()
                />

                <div class="divider my-0"></div>

                // Credit points filter
                <CreditFilter
                    filters=filters
                    filter_options=filter_options_ok.into()
                />

                <div class="divider my-0"></div>

                // Semester rotation filter
                <SemesterFilter
                    filters=filters
                    filter_options=filter_options_ok.into()
                />

                <div class="divider my-0"></div>

                // Exam type filter
                <ExamFilter
                    filters=filters
                    filter_options=filter_options_ok.into()
                />

                <div class="divider my-0"></div>

                // Component type filter
                <ComponentTypeFilter
                    filters=filters
                    filter_options=filter_options_ok.into()
                />

                <div class="divider my-0"></div>

                // Component language filter
                <ComponentLanguageFilter
                    filters=filters
                    filter_options=filter_options_ok.into()
                />
            </div>
        </div>
    }
}
