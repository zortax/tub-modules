use crate::models::{FilterOptions, SearchFilters};
use leptos::prelude::*;

#[component]
pub fn ExamFilter(
    filters: RwSignal<SearchFilters>,
    filter_options: Signal<Option<FilterOptions>>,
) -> impl IntoView {
    view! {
        <div class="form-control">
            <label class="label pb-1">
                <span class="label-text font-semibold text-base flex items-center gap-2">
                    <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 20 20" fill="currentColor">
                        <path d="M9 2a1 1 0 000 2h2a1 1 0 100-2H9z" />
                        <path fill-rule="evenodd" d="M4 5a2 2 0 012-2 3 3 0 003 3h2a3 3 0 003-3 2 2 0 012 2v11a2 2 0 01-2 2H6a2 2 0 01-2-2V5zm9.707 5.707a1 1 0 00-1.414-1.414L9 12.586l-1.293-1.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clip-rule="evenodd" />
                    </svg>
                    "Exam Type"
                </span>
            </label>
            <div class="flex flex-wrap gap-2 mt-2">
                {move || {
                    filter_options.get()
                        .map(|opts| {
                            opts.exam_categories
                                .iter()
                                .map(|category| {
                                    let category_for_check = category.clone();
                                    let category_for_change = category.clone();
                                    let category_label = category.clone();
                                    let is_checked = move || {
                                        filters.get()
                                            .exam_categories
                                            .as_ref()
                                            .map(|cats| cats.contains(&category_for_check))
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
                                                    let category = category_for_change.clone();
                                                    let mut new_filters = filters.get();
                                                    if checked {
                                                        let mut categories = new_filters.exam_categories.unwrap_or_default();
                                                        if !categories.contains(&category) {
                                                            categories.push(category);
                                                        }
                                                        new_filters.exam_categories = Some(categories);
                                                    } else {
                                                        if let Some(mut categories) = new_filters.exam_categories {
                                                            categories.retain(|c| c != &category);
                                                            new_filters.exam_categories = if categories.is_empty() { None } else { Some(categories) };
                                                        }
                                                    }
                                                    filters.set(new_filters);
                                                }
                                            />
                                            <span class="label-text text-sm">{category_label}</span>
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
