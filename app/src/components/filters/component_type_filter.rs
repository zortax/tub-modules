use crate::models::{FilterOptions, SearchFilters};
use leptos::prelude::*;

#[component]
pub fn ComponentTypeFilter(
    filters: RwSignal<SearchFilters>,
    filter_options: Signal<Option<FilterOptions>>,
) -> impl IntoView {
    view! {
        <div class="form-control">
            <label class="label pb-1">
                <span class="label-text font-semibold text-base flex items-center gap-2">
                    <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 20 20" fill="currentColor">
                        <path d="M7 3a1 1 0 000 2h6a1 1 0 100-2H7zM4 7a1 1 0 011-1h10a1 1 0 110 2H5a1 1 0 01-1-1zM2 11a2 2 0 012-2h12a2 2 0 012 2v4a2 2 0 01-2 2H4a2 2 0 01-2-2v-4z" />
                    </svg>
                    "Component Type"
                </span>
            </label>
            <div class="flex flex-wrap gap-2 mt-2">
                {move || {
                    filter_options.get()
                        .map(|opts| {
                            opts.component_types
                                .iter()
                                .map(|comp_type| {
                                    let type_for_check = comp_type.clone();
                                    let type_for_change = comp_type.clone();
                                    let type_label = comp_type.clone();
                                    let is_checked = move || {
                                        filters.get()
                                            .component_types
                                            .as_ref()
                                            .map(|types| types.contains(&type_for_check))
                                            .unwrap_or(false)
                                    };
                                    view! {
                                        <label class="label cursor-pointer justify-start gap-2 border border-base-300 rounded-lg px-3 py-2 hover:bg-base-200 transition-colors">
                                            <input
                                                type="checkbox"
                                                class="checkbox checkbox-ghost checkbox-sm"
                                                prop:checked=is_checked
                                                on:change=move |ev| {
                                                    let checked = event_target_checked(&ev);
                                                    let comp_type = type_for_change.clone();
                                                    let mut new_filters = filters.get();
                                                    if checked {
                                                        let mut types = new_filters.component_types.unwrap_or_default();
                                                        if !types.contains(&comp_type) {
                                                            types.push(comp_type);
                                                        }
                                                        new_filters.component_types = Some(types);
                                                    } else {
                                                        if let Some(mut types) = new_filters.component_types {
                                                            types.retain(|t| t != &comp_type);
                                                            new_filters.component_types = if types.is_empty() { None } else { Some(types) };
                                                        }
                                                    }
                                                    filters.set(new_filters);
                                                }
                                            />
                                            <span class="label-text text-sm">{type_label}</span>
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
