use leptos::prelude::*;

#[component]
pub fn ActionsBar(
    moses_link: String,
    on_close: Option<Callback<()>>,
) -> impl IntoView {
    view! {
        <div class="flex items-center justify-between gap-3 pb-3 border-b border-base-300/50">
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
            {if let Some(close_fn) = on_close {
                view! {
                    <button
                        class="btn btn-sm btn-ghost btn-circle hover:bg-base-200"
                        on:click=move |_| close_fn.run(())
                        title="Close (Esc)"
                    >
                        <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                        </svg>
                    </button>
                }.into_any()
            } else {
                view! { <></> }.into_any()
            }}
        </div>
    }
}
