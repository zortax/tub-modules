use crate::components::shared::ComponentTable;
use crate::models::ModuleSummary;
use leptos::prelude::*;

#[component]
pub fn ModuleCard(
    module: ModuleSummary,
    is_starred: Signal<bool>,
    on_toggle_star: impl Fn() + 'static,
    on_click: impl Fn() + 'static,
) -> impl IntoView {
    let _module_id = module.id;
    let _module_version = module.version;

    view! {
        <div
            class="card bg-base-100 shadow-md hover:shadow-lg transition-shadow cursor-pointer"
            on:click=move |_ev| {
                // Check if Ctrl/Cmd key is pressed
                #[cfg(target_arch = "wasm32")]
                {
                    if _ev.ctrl_key() || _ev.meta_key() {
                        // Open in new tab
                        if let Some(window) = web_sys::window() {
                            let url = format!("/module/{}/{}", _module_id, _module_version);
                            let _ = window.open_with_url_and_target(&url, "_blank");
                        }
                    } else {
                        // Normal click - open modal
                        on_click();
                    }
                }
                #[cfg(not(target_arch = "wasm32"))]
                {
                    let _ = _ev;  // Silence unused warning
                    on_click();
                }
            }
        >
            <div class="card-body p-4 gap-2">
                <div class="flex justify-between items-start gap-3">
                    <div class="flex-1 flex items-baseline gap-2 flex-wrap">
                        <h3 class="card-title text-xl leading-tight">
                            {module.title.clone()}
                        </h3>
                        <div class="tooltip tooltip-bottom" data-tip="Open in new tab">
                            <button
                                class="btn btn-ghost btn-xs btn-circle opacity-50 hover:opacity-100 transition-opacity"
                                on:click=move |ev| {
                                    ev.stop_propagation();
                                    #[cfg(target_arch = "wasm32")]
                                    {
                                        if let Some(window) = web_sys::window() {
                                            let url = format!("/module/{}/{}", _module_id, _module_version);
                                            let _ = window.open_with_url_and_target(&url, "_blank");
                                        }
                                    }
                                }
                            >
                                <svg xmlns="http://www.w3.org/2000/svg" class="h-3.5 w-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14" />
                                </svg>
                            </button>
                        </div>
                        <span class="text-sm font-normal text-base-content/40 whitespace-nowrap">
                            "Module " {module.id} " v" {module.version}
                        </span>
                    </div>
                    <div class="flex items-center gap-2 shrink-0">
                        <div class="tooltip tooltip-left" attr:data-tip=move || if is_starred.get() { "Unstar module" } else { "Star module" }>
                            <button
                                class="btn btn-ghost btn-sm btn-circle"
                                on:click=move |ev| {
                                    ev.stop_propagation();
                                    on_toggle_star();
                                }
                            >
                                {move || if is_starred.get() {
                                    view! {
                                        <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
                                            <path d="M9.049 2.927c.3-.921 1.603-.921 1.902 0l1.07 3.292a1 1 0 00.95.69h3.462c.969 0 1.371 1.24.588 1.81l-2.8 2.034a1 1 0 00-.364 1.118l1.07 3.292c.3.921-.755 1.688-1.54 1.118l-2.8-2.034a1 1 0 00-1.175 0l-2.8 2.034c-.784.57-1.838-.197-1.539-1.118l1.07-3.292a1 1 0 00-.364-1.118L2.98 8.72c-.783-.57-.38-1.81.588-1.81h3.461a1 1 0 00.951-.69l1.07-3.292z" />
                                        </svg>
                                    }.into_any()
                                } else {
                                    view! {
                                        <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11.049 2.927c.3-.921 1.603-.921 1.902 0l1.519 4.674a1 1 0 00.95.69h4.915c.969 0 1.371 1.24.588 1.81l-3.976 2.888a1 1 0 00-.363 1.118l1.518 4.674c.3.922-.755 1.688-1.538 1.118l-3.976-2.888a1 1 0 00-1.176 0l-3.976 2.888c-.783.57-1.838-.197-1.538-1.118l1.518-4.674a1 1 0 00-.363-1.118l-3.976-2.888c-.784-.57-.38-1.81.588-1.81h4.914a1 1 0 00.951-.69l1.519-4.674z" />
                                        </svg>
                                    }.into_any()
                                }}
                            </button>
                        </div>
                        <div class="flex items-center gap-1.5">
                            {module.languages.iter().map(|lang| {
                                let lang = lang.clone();
                                view! {
                                    <div class="badge badge-soft badge-accent">
                                        {lang}
                                    </div>
                                }
                            }).collect_view()}
                            <div class="badge badge-soft badge-primary">
                                {module.credits} " CP"
                            </div>
                        </div>
                    </div>
                </div>

                {if !module.faculty_name.is_empty() || !module.exam_categories.is_empty() {
                    view! {
                        <div class="flex items-center gap-2 text-base-content/60">
                            {if !module.faculty_name.is_empty() {
                                view! {
                                    <span>"Faculty " {module.faculty_name.clone()}</span>
                                }.into_any()
                            } else {
                                view! { <></> }.into_any()
                            }}
                            {if !module.faculty_name.is_empty() && !module.exam_categories.is_empty() {
                                view! {
                                    <span class="text-base-content/30">"|"</span>
                                }.into_any()
                            } else {
                                view! { <></> }.into_any()
                            }}
                            {if !module.exam_categories.is_empty() {
                                view! {
                                    <div class="flex flex-wrap gap-1.5">
                                        {module.exam_categories.iter().map(|category| {
                                            let category = category.clone();
                                            view! {
                                                <div class="badge badge-soft badge-accent">
                                                    {category}
                                                </div>
                                            }
                                        }).collect_view()}
                                    </div>
                                }.into_any()
                            } else {
                                view! { <></> }.into_any()
                            }}
                        </div>
                    }.into_any()
                } else {
                    view! { <></> }.into_any()
                }}

                // Study programs
                {if !module.study_programs.is_empty() {
                    let program_count = module.study_programs.len();
                    view! {
                        <div class="flex items-center gap-2 text-sm text-base-content/60">
                            <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 20 20" fill="currentColor">
                                <path d="M10.394 2.08a1 1 0 00-.788 0l-7 3a1 1 0 000 1.84L5.25 8.051a.999.999 0 01.356-.257l4-1.714a1 1 0 11.788 1.838L7.667 9.088l1.94.831a1 1 0 00.787 0l7-3a1 1 0 000-1.838l-7-3zM3.31 9.397L5 10.12v4.102a8.969 8.969 0 00-1.05-.174 1 1 0 01-.89-.89 11.115 11.115 0 01.25-3.762zM9.3 16.573A9.026 9.026 0 007 14.935v-3.957l1.818.78a3 3 0 002.364 0l5.508-2.361a11.026 11.026 0 01.25 3.762 1 1 0 01-.89.89 8.968 8.968 0 00-5.35 2.524 1 1 0 01-1.4 0zM6 18a1 1 0 001-1v-2.065a8.935 8.935 0 00-2-.712V17a1 1 0 001 1z" />
                            </svg>
                            {if program_count == 1 {
                                view! { <span>{module.study_programs[0].clone()}</span> }.into_any()
                            } else {
                                view! {
                                    <span>{program_count} " study programs"</span>
                                }.into_any()
                            }}
                        </div>
                    }.into_any()
                } else {
                    view! { <></> }.into_any()
                }}

                // Module components - prevent click propagation
                <div class="mt-3" on:click=|ev| ev.stop_propagation()>
                    <ComponentTable components=module.components.clone() />
                </div>
            </div>
        </div>
    }
}
