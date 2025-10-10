use leptos::prelude::*;

#[component]
pub fn ModuleDetailHeader(
    title: String,
    id: i32,
    version: i32,
    credits: i32,
    languages: Vec<String>,
) -> impl IntoView {
    view! {
        <div class="flex flex-col gap-3 pt-3">
            <div class="flex items-baseline gap-3 flex-wrap">
                <h1 class="text-2xl font-bold leading-tight text-base-content">{title}</h1>
                <span class="text-sm text-base-content/50 whitespace-nowrap">
                    "Module " {id} " · v" {version}
                </span>
            </div>
            <div class="flex items-center gap-2 flex-wrap">
                <div class="badge badge-soft badge-primary badge-lg font-semibold">
                    {credits} " ECTS"
                </div>
                {languages.into_iter().map(|lang| {
                    view! {
                        <div class="badge badge-soft badge-secondary">
                            {lang}
                        </div>
                    }
                }).collect_view()}
            </div>
        </div>
    }
}
