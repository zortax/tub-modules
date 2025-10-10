use crate::models::{FilterOptions, SearchFilters};
use leptos::prelude::*;

#[component]
pub fn ProgramFilter(
    filters: RwSignal<SearchFilters>,
    filter_options: Signal<Option<FilterOptions>>,
) -> impl IntoView {
    let search_query = RwSignal::new(String::new());

    view! {
        <div class="form-control">
            <label class="label pb-1">
                <span class="label-text font-semibold text-base flex items-center gap-2">
                    <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 20 20" fill="currentColor">
                        <path d="M10.394 2.08a1 1 0 00-.788 0l-7 3a1 1 0 000 1.84L5.25 8.051a.999.999 0 01.356-.257l4-1.714a1 1 0 11.788 1.838L7.667 9.088l1.94.831a1 1 0 00.787 0l7-3a1 1 0 000-1.838l-7-3zM3.31 9.397L5 10.12v4.102a8.969 8.969 0 00-1.05-.174 1 1 0 01-.89-.89 11.115 11.115 0 01.25-3.762zM9.3 16.573A9.026 9.026 0 007 14.935v-3.957l1.818.78a3 3 0 002.364 0l5.508-2.361a11.026 11.026 0 01.25 3.762 1 1 0 01-.89.89 8.968 8.968 0 00-5.35 2.524 1 1 0 01-1.4 0zM6 18a1 1 0 001-1v-2.065a8.935 8.935 0 00-2-.712V17a1 1 0 001 1z" />
                    </svg>
                    "Study Programs"
                </span>
            </label>

            <input
                type="text"
                placeholder="Search programs..."
                class="input input-bordered input-sm w-full mb-2"
                prop:value=move || search_query.get()
                on:input=move |ev| {
                    search_query.set(event_target_value(&ev));
                }
            />

            <div class="flex flex-col gap-1 max-h-48 overflow-y-auto overflow-x-hidden border border-base-300 rounded-lg p-2 bg-base-200/30">
                {move || {
                    let query = search_query.get().to_lowercase();
                    filter_options.get()
                        .map(|opts| {
                            opts.study_programs
                                .iter()
                                .filter(|program| {
                                    if query.is_empty() {
                                        true
                                    } else {
                                        program.name.to_lowercase().contains(&query)
                                    }
                                })
                                .map(|program| {
                                    let program_id = program.id;
                                    let program_name = program.name.clone();
                                    let is_checked = move || {
                                        filters.get()
                                            .study_program_ids
                                            .as_ref()
                                            .map(|ids| ids.contains(&program_id))
                                            .unwrap_or(false)
                                    };
                                    view! {
                                        <label class="flex items-start cursor-pointer gap-2 py-1 px-2 hover:bg-base-200 rounded w-full">
                                            <input
                                                type="checkbox"
                                                class="checkbox checkbox-ghost checkbox-sm shrink-0 mt-0.5"
                                                prop:checked=is_checked
                                                on:change=move |ev| {
                                                    let checked = event_target_checked(&ev);
                                                    let mut new_filters = filters.get();
                                                    if checked {
                                                        let mut ids = new_filters.study_program_ids.unwrap_or_default();
                                                        if !ids.contains(&program_id) {
                                                            ids.push(program_id);
                                                        }
                                                        new_filters.study_program_ids = Some(ids);
                                                    } else {
                                                        if let Some(mut ids) = new_filters.study_program_ids {
                                                            ids.retain(|&id| id != program_id);
                                                            new_filters.study_program_ids = if ids.is_empty() { None } else { Some(ids) };
                                                        }
                                                    }
                                                    filters.set(new_filters);
                                                }
                                            />
                                            <span class="label-text text-sm break-words flex-1">{program_name}</span>
                                        </label>
                                    }
                                })
                                .collect_view()
                        })
                        .unwrap_or_else(|| vec![])
                }}
            </div>
        </div>
    }
}
