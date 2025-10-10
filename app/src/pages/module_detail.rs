use crate::api::get_module_detail;
use crate::components::layout::PageLayout;
use crate::components::module_detail::ModuleDetailView;
use leptos::prelude::*;
use leptos_router::hooks::use_params;
use leptos_router::params::Params;

#[derive(Params, PartialEq, Clone)]
struct ModuleParams {
    id: i32,
    version: i32,
}

#[component]
pub fn ModuleDetailPage() -> impl IntoView {
    let params = use_params::<ModuleParams>();

    let module_data = Resource::new(
        move || params.get(),
        |params_result| async move {
            match params_result {
                Ok(p) => get_module_detail(p.id, p.version).await,
                Err(e) => Err(ServerFnError::new(e.to_string())),
            }
        },
    );

    view! {
        <PageLayout>
            <div class="py-8 max-w-5xl mx-auto">
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
                                    <ModuleDetailView module=module on_close=None />
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
        </PageLayout>
    }
}
