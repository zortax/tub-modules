use crate::components::shared::ComponentTable;
use crate::models::ComponentInfo;
use leptos::prelude::*;

#[component]
pub fn ComponentsSection(components: Vec<ComponentInfo>) -> impl IntoView {
    if components.is_empty() {
        return view! { <></> }.into_any();
    }

    view! {
        <div class="card bg-base-100 shadow-sm">
            <div class="card-body p-5">
                <h2 class="card-title text-lg text-primary mb-1">Module Components</h2>
                <div class="divider my-0 opacity-50"></div>
                <div class="mt-3">
                    <ComponentTable components=components />
                </div>
            </div>
        </div>
    }
    .into_any()
}
