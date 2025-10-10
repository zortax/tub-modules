use leptos::prelude::*;

#[component]
pub fn ModalHeader(
    title: String,
    id: i32,
    version: i32,
    credits: i32,
    languages: Vec<String>,
    moses_link: String,
    on_close: Callback<()>,
) -> impl IntoView {
    view! {
        <div class="flex flex-col gap-3">
            // Top row: title and buttons
            <div class="flex items-center justify-between gap-4 pb-3 border-b border-base-300/50">
                <div class="flex items-baseline gap-3 flex-wrap flex-1 min-w-0">
                    <h1 class="text-2xl font-bold leading-tight text-base-content truncate">{title}</h1>
                    <span class="text-sm text-base-content/50 whitespace-nowrap">
                        "Module " {id} " · v" {version}
                    </span>
                </div>
                <div class="flex items-center gap-2 shrink-0">
                    <a
                        href=moses_link
                        target="_blank"
                        rel="noopener noreferrer"
                        class="btn btn-sm btn-soft btn-primary gap-2"
                    >
                        <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14" />
                        </svg>
                        "View on Moses"
                    </a>
                    <div class="tooltip tooltip-left" data-tip="Close (Esc)">
                        <button
                            class="btn btn-sm btn-ghost btn-circle hover:bg-base-200"
                            on:click=move |_| on_close.run(())
                        >
                            <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                            </svg>
                        </button>
                    </div>
                </div>
            </div>

            // Bottom row: badges
            <div class="flex items-center gap-2 flex-wrap">
                <div class="badge badge-soft badge-primary badge-lg font-semibold">
                    {credits} " CP"
                </div>
                {languages.into_iter().map(|lang| {
                    view! {
                        <div class="badge badge-soft badge-accent">
                            {lang}
                        </div>
                    }
                }).collect_view()}
            </div>
        </div>
    }
}
