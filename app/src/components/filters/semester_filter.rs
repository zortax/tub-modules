use crate::models::{FilterOptions, SearchFilters};
use leptos::prelude::*;

#[component]
pub fn SemesterFilter(
    filters: RwSignal<SearchFilters>,
    filter_options: Signal<Option<FilterOptions>>,
) -> impl IntoView {
    view! {
        <div class="form-control">
            <label class="label pb-1">
                <span class="label-text font-semibold text-base flex items-center gap-2">
                    <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 20 20" fill="currentColor">
                        <path fill-rule="evenodd" d="M4 2a1 1 0 011 1v2.101a7.002 7.002 0 0111.601 2.566 1 1 0 11-1.885.666A5.002 5.002 0 005.999 7H9a1 1 0 010 2H4a1 1 0 01-1-1V3a1 1 0 011-1zm.008 9.057a1 1 0 011.276.61A5.002 5.002 0 0014.001 13H11a1 1 0 110-2h5a1 1 0 011 1v5a1 1 0 11-2 0v-2.101a7.002 7.002 0 01-11.601-2.566 1 1 0 01.61-1.276z" clip-rule="evenodd" />
                    </svg>
                    "Semester Rotation"
                </span>
            </label>
            <div class="flex flex-wrap gap-2 mt-2">
                {move || {
                    filter_options.get()
                        .map(|opts| {
                            opts.semester_rotations
                                .iter()
                                .map(|rotation| {
                                    let rotation_for_check = rotation.clone();
                                    let rotation_for_change = rotation.clone();
                                    let rotation_label = rotation.clone();
                                    let is_checked = move || {
                                        filters.get()
                                            .semester_rotations
                                            .as_ref()
                                            .map(|rots| rots.contains(&rotation_for_check))
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
                                                    let rotation = rotation_for_change.clone();
                                                    let mut new_filters = filters.get();
                                                    if checked {
                                                        let mut rotations = new_filters.semester_rotations.unwrap_or_default();
                                                        if !rotations.contains(&rotation) {
                                                            rotations.push(rotation);
                                                        }
                                                        new_filters.semester_rotations = Some(rotations);
                                                    } else {
                                                        if let Some(mut rotations) = new_filters.semester_rotations {
                                                            rotations.retain(|r| r != &rotation);
                                                            new_filters.semester_rotations = if rotations.is_empty() { None } else { Some(rotations) };
                                                        }
                                                    }
                                                    filters.set(new_filters);
                                                }
                                            />
                                            <span class="label-text text-sm">{rotation_label}</span>
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
