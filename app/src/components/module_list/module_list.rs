use crate::components::module_list::ModuleCard;
use crate::models::ModuleSummary;
use crate::starred::{is_starred, toggle_starred, StarredModules};
use leptos::prelude::*;

#[component]
pub fn ModuleList(
    modules: RwSignal<Vec<ModuleSummary>>,
    starred: Signal<StarredModules>,
    set_starred: WriteSignal<StarredModules>,
    on_module_click: impl Fn(i32, i32) + 'static + Copy + Send,
    #[prop(optional)] is_loading: Option<Signal<bool>>,
) -> impl IntoView {
    view! {
        <div class="space-y-4">
            {move || {
                let loading = is_loading.map(|s| s.get()).unwrap_or(false);
                let is_empty = modules.with(|m| m.is_empty());

                if is_empty && !loading {
                    view! {
                        <div class="card bg-base-100 shadow-xl">
                            <div class="card-body items-center text-center">
                                <h3 class="card-title">"No modules found"</h3>
                                <p class="text-base-content/70">
                                    "Try adjusting your filters or search query."
                                </p>
                            </div>
                        </div>
                    }.into_any()
                } else {
                    view! {
                        <For
                            each=move || modules.get()
                            key=|module| (module.id, module.version)
                            children=move |module| {
                                let module_id = module.id;
                                let module_version = module.version;
                                let is_starred_signal = Signal::derive(move || {
                                    is_starred(&starred.get(), module_id, module_version)
                                });
                                let on_toggle = move || {
                                    set_starred.update(|s| {
                                        *s = toggle_starred(s, module_id, module_version);
                                    });
                                };

                                view! {
                                    <div class="animate-fade-in">
                                        <ModuleCard
                                            module=module
                                            is_starred=is_starred_signal
                                            on_toggle_star=on_toggle
                                            on_click=move || on_module_click(module_id, module_version)
                                        />
                                    </div>
                                }
                            }
                        />
                    }.into_any()
                }
            }}
        </div>
    }
}
