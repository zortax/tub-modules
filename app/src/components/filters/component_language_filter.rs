use crate::models::{FilterOptions, SearchFilters};
use leptos::prelude::*;

#[component]
pub fn ComponentLanguageFilter(
    filters: RwSignal<SearchFilters>,
    filter_options: Signal<Option<FilterOptions>>,
) -> impl IntoView {
    view! {
        <div class="form-control">
            <label class="label pb-1">
                <span class="label-text font-semibold text-base flex items-center gap-2">
                    <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 20 20" fill="currentColor">
                        <path fill-rule="evenodd" d="M7 2a1 1 0 011 1v1h3a1 1 0 110 2H9.578a18.87 18.87 0 01-1.724 4.78c.29.354.596.696.914 1.026a1 1 0 11-1.44 1.389c-.188-.196-.373-.396-.554-.6a19.098 19.098 0 01-3.107 3.567 1 1 0 01-1.334-1.49 17.087 17.087 0 003.13-3.733 18.992 18.992 0 01-1.487-2.494 1 1 0 111.79-.89c.234.47.489.928.764 1.372.417-.934.752-1.913.997-2.927H3a1 1 0 110-2h3V3a1 1 0 011-1zm6 6a1 1 0 01.894.553l2.991 5.982a.869.869 0 01.02.037l.99 1.98a1 1 0 11-1.79.895L15.383 16h-4.764l-.724 1.447a1 1 0 11-1.788-.894l.99-1.98.019-.038 2.99-5.982A1 1 0 0113 8zm-1.382 6h2.764L13 11.236 11.618 14z" clip-rule="evenodd" />
                    </svg>
                    "Component Language"
                </span>
            </label>
            <div class="flex flex-wrap gap-2 mt-2">
                {move || {
                    filter_options.get()
                        .map(|opts| {
                            opts.component_languages
                                .iter()
                                .map(|language| {
                                    let lang_for_check = language.clone();
                                    let lang_for_change = language.clone();
                                    let lang_label = language.clone();
                                    let is_checked = move || {
                                        filters.get()
                                            .component_languages
                                            .as_ref()
                                            .map(|langs| langs.contains(&lang_for_check))
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
                                                    let language = lang_for_change.clone();
                                                    let mut new_filters = filters.get();
                                                    if checked {
                                                        let mut languages = new_filters.component_languages.unwrap_or_default();
                                                        if !languages.contains(&language) {
                                                            languages.push(language);
                                                        }
                                                        new_filters.component_languages = Some(languages);
                                                    } else {
                                                        if let Some(mut languages) = new_filters.component_languages {
                                                            languages.retain(|l| l != &language);
                                                            new_filters.component_languages = if languages.is_empty() { None } else { Some(languages) };
                                                        }
                                                    }
                                                    filters.set(new_filters);
                                                }
                                            />
                                            <span class="label-text text-sm">{lang_label}</span>
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
