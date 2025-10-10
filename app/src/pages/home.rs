use crate::api::{get_filter_options, get_module_count, search_modules_paginated};
use crate::components::{FilterPanel, ModuleDetailModal, ModuleList, PageLayout};
use crate::models::ModuleSummary;
use crate::starred::{is_starred, use_search_filters, use_starred_modules};
use leptos::prelude::*;
use leptos::task::spawn_local;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;

const PAGE_SIZE: i64 = 100;

#[component]
pub fn HomePage() -> impl IntoView {
    // State for filters (stored in local storage)
    let filters = use_search_filters();

    // Starred modules (stored in local storage)
    let (starred, set_starred) = use_starred_modules();

    // Modal state for module details
    let selected_module = RwSignal::new(None::<(i32, i32)>);

    // Load filter options on mount (SSR)
    let filter_options = Resource::new(|| (), |_| async { get_filter_options().await });

    // Accumulated modules (all fetched from server)
    let all_modules = RwSignal::new(Vec::<ModuleSummary>::new());

    // Filtered modules (client-side filtering for starred)
    let modules = RwSignal::new(Vec::<ModuleSummary>::new());

    // Update filtered modules when all_modules or starred filter changes
    Effect::new(move || {
        let mods = all_modules.get();
        let starred_only = filters.get().starred_only;

        let filtered = if starred_only {
            let starred_data = starred.get();
            mods.into_iter()
                .filter(|m| is_starred(&starred_data, m.id, m.version))
                .collect()
        } else {
            mods
        };

        modules.set(filtered);
    });

    let is_loading = RwSignal::new(false);
    let has_more = RwSignal::new(true);
    let total_count = RwSignal::new(0_i64);

    // Load next page
    let load_next_page = move || {
        spawn_local(async move {
            if is_loading.get_untracked() || !has_more.get_untracked() {
                return;
            }

            is_loading.set(true);
            let current_filters = filters.get_untracked();
            let current_page = (all_modules.with_untracked(|m| m.len()) as i64) / PAGE_SIZE;

            match search_modules_paginated(current_filters, current_page, PAGE_SIZE).await {
                Ok(new_modules) => {
                    let count = new_modules.len();

                    // Update all_modules
                    all_modules.update(|m| m.extend(new_modules));

                    if count < PAGE_SIZE as usize {
                        has_more.set(false);
                    }

                    // Add a small delay to let browser render before next load
                    if count == PAGE_SIZE as usize {
                        gloo_timers::future::TimeoutFuture::new(100).await;
                    }

                    is_loading.set(false);
                }
                Err(e) => {
                    leptos::logging::error!("Error loading modules: {:?}", e);
                    is_loading.set(false);
                }
            }
        });
    };

    // Fetch count and reset when filters change (excluding starred_only which is client-side)
    Effect::new(move || {
        let current_filters = filters.get();

        // Only reset and fetch if non-starred filters changed
        // starred_only is handled client-side via the memo
        all_modules.set(Vec::new());
        has_more.set(true);
        is_loading.set(false);

        // Fetch total count
        spawn_local(async move {
            match get_module_count(current_filters.clone()).await {
                Ok(count) => total_count.set(count),
                Err(e) => leptos::logging::error!("Error fetching count: {:?}", e),
            }
        });

        // Load first page
        load_next_page();
    });

    // Scroll listener to load more
    Effect::new(move || {
        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::closure::Closure;

            let window = web_sys::window().expect("no global window");

            let closure = Closure::wrap(Box::new(move || {
                let window = web_sys::window().expect("no global window");
                let document = window.document().expect("no document");

                if let Some(body) = document.body() {
                    let scroll_top = window.scroll_y().unwrap_or(0.0);
                    let window_height = window.inner_height().unwrap().as_f64().unwrap_or(0.0);
                    let scroll_height = body.scroll_height() as f64;

                    // Load more when within 1000px of bottom
                    if scroll_top + window_height > scroll_height - 1000.0 {
                        let loading = is_loading.get_untracked();
                        let more = has_more.get_untracked();

                        if !loading && more {
                            load_next_page();
                        }
                    }
                }
            }) as Box<dyn Fn()>);

            window
                .add_event_listener_with_callback("scroll", closure.as_ref().unchecked_ref())
                .expect("failed to add scroll listener");

            closure.forget();
        }
    });

    view! {
        <PageLayout>
            <div class="grid grid-cols-1 lg:grid-cols-[380px_1fr] gap-6">
                // Sidebar - hidden on mobile, shown in drawer
                <aside class="hidden lg:block">
                    <div class="sticky top-[7rem] self-start">
                        <Suspense fallback=move || view! {
                            <div class="card bg-base-100 shadow-xl">
                                <div class="card-body">
                                    <div class="flex justify-center">
                                        <span class="loading loading-spinner loading-md"></span>
                                    </div>
                                </div>
                            </div>
                        }>
                            {move || view! {
                                <FilterPanel
                                    filters=filters
                                    filter_options=filter_options.get().into()
                                />
                            }}
                        </Suspense>
                    </div>
                </aside>

                // Mobile drawer for filters
                <div class="lg:hidden mb-4">
                    <label for="filter-drawer" class="btn btn-primary drawer-button">
                        <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5 mr-2" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 4a1 1 0 011-1h16a1 1 0 011 1v2.586a1 1 0 01-.293.707l-6.414 6.414a1 1 0 00-.293.707V17l-4 4v-6.586a1 1 0 00-.293-.707L3.293 7.293A1 1 0 013 6.586V4z" />
                        </svg>
                        "Filters"
                    </label>
                </div>

                // Main content area with max width
                <main class="w-full max-w-4xl mx-auto lg:mx-0 relative">
                    // Sticky header with module count and loading indicator
                    <div class="sticky top-[5.5rem] z-10 pb-4 mb-4">
                        // Solid background behind text - extends upward beyond element bounds to cover all gaps
                        <div class="absolute inset-x-0 -top-8 bottom-4 bg-base-300"></div>
                        // Bottom gradient layer - fade into module cards
                        <div class="absolute inset-x-0 bottom-0 h-4 bg-gradient-to-b from-base-300 to-transparent"></div>

                        // Content
                        <div class="relative pt-4 flex items-center justify-between">
                            {move || {
                                let loaded = modules.with(|m| m.len());
                                let total = total_count.get();
                                let loading = is_loading.get();

                                view! {
                                    <div class="flex items-center gap-2">
                                        <span class="text-sm text-base-content/60 font-medium">
                                            {if total > 0 && loaded > 0 {
                                                if loaded < total as usize {
                                                    format!("Loaded {} of {} modules", loaded, total)
                                                } else {
                                                    format!("Showing all {} modules", total)
                                                }
                                            } else if total == 0 && !loading {
                                                "No modules found".to_string()
                                            } else if loading && loaded == 0 {
                                                "Loading modules...".to_string()
                                            } else {
                                                format!("Found {} module{}", total, if total == 1 { "" } else { "s" })
                                            }}
                                        </span>
                                        {move || {
                                            if loading {
                                                view! {
                                                    <span class="loading loading-spinner loading-xs"></span>
                                                }.into_any()
                                            } else {
                                                view! { <></> }.into_any()
                                            }
                                        }}
                                    </div>
                                }
                            }}
                        </div>
                    </div>

                    // Module list
                    <ModuleList
                        modules=modules
                        starred=starred
                        set_starred=set_starred
                        on_module_click=move |id, version| selected_module.set(Some((id, version)))
                        is_loading=is_loading.into()
                    />
                </main>
            </div>

            // Drawer for mobile filters
            <div class="drawer drawer-start lg:hidden">
                <input id="filter-drawer" type="checkbox" class="drawer-toggle" />
                <div class="drawer-side z-50">
                    <label for="filter-drawer" class="drawer-overlay"></label>
                    <div class="bg-base-100 w-80 min-h-full p-4">
                        <Suspense fallback=move || view! {
                            <div class="card bg-base-100 shadow-xl">
                                <div class="card-body">
                                    <div class="flex justify-center">
                                        <span class="loading loading-spinner loading-md"></span>
                                    </div>
                                </div>
                            </div>
                        }>
                            {move || view! {
                                <FilterPanel
                                    filters=filters
                                    filter_options=filter_options.get().into()
                                />
                            }}
                        </Suspense>
                    </div>
                </div>
            </div>

            // Module detail modal
            {move || {
                if let Some((id, version)) = selected_module.get() {
                    view! {
                        <ModuleDetailModal
                            module_id=id
                            module_version=version
                            on_close=Callback::new(move |_| selected_module.set(None))
                        />
                    }.into_any()
                } else {
                    view! { <></> }.into_any()
                }
            }}
        </PageLayout>
    }
}
