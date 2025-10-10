use crate::api::get_module_detail;
use crate::components::module_detail::ModuleDetailView;
use leptos::prelude::*;

#[component]
pub fn ModuleDetailModal(
    module_id: i32,
    module_version: i32,
    on_close: Callback<()>,
) -> impl IntoView {
    let module_data = Resource::new(
        move || (module_id, module_version),
        |(id, version)| async move { get_module_detail(id, version).await },
    );

    // Handle ESC key
    #[cfg(target_arch = "wasm32")]
    {
        use leptos::ev::{self, KeyboardEvent};

        leptos::prelude::window_event_listener(ev::keydown, move |evt: KeyboardEvent| {
            if evt.key() == "Escape" {
                on_close.run(());
            }
        });
    }

    view! {
        <div
            class="fixed inset-0 z-50 flex items-start justify-center p-4 bg-black/60 backdrop-blur-sm overflow-y-auto animate-in fade-in duration-200"
            on:click=move |_| on_close.run(())
        >
            <div
                class="relative w-full max-w-4xl my-8 bg-base-200 rounded-lg shadow-2xl animate-in fade-in zoom-in-95 duration-200"
                on:click=|ev| ev.stop_propagation()
            >
                <div class="p-6">
                    <Suspense fallback=move || {
                        view! {
                            <div class="flex items-center justify-center py-12">
                                <span class="loading loading-spinner loading-lg"></span>
                            </div>
                        }
                    }>
                        {move || Suspend::new(async move {
                            match module_data.await {
                                Ok(module) => {
                                    view! {
                                        <ModuleDetailView
                                            module=module
                                            on_close=Some(on_close)
                                        />
                                    }
                                    .into_any()
                                }
                                Err(e) => {
                                    view! {
                                        <div class="alert alert-error">
                                            <span>"Error loading module details: " {e.to_string()}</span>
                                        </div>
                                    }
                                    .into_any()
                                }
                            }
                        })}
                    </Suspense>
                </div>
            </div>
        </div>
    }
}
