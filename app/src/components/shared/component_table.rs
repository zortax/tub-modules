use crate::models::ComponentInfo;
use leptos::prelude::*;

#[component]
pub fn ComponentTable(components: Vec<ComponentInfo>) -> impl IntoView {
    if components.is_empty() {
        return view! { <></> }.into_any();
    }

    view! {
        <div class="overflow-hidden rounded-lg">
            <table class="w-full text-sm" style="border-spacing: 0 1px; border-collapse: separate;">
                <thead>
                    <tr class="bg-base-200/30">
                        <th class="text-left px-2 py-1 font-medium text-base-content/40">Type</th>
                        <th class="text-left px-2 py-1 font-medium text-base-content/40">Name</th>
                        <th class="text-left px-2 py-1 font-medium text-base-content/40">Number</th>
                        <th class="text-left px-2 py-1 font-medium text-base-content/40">Rotation</th>
                        <th class="text-left px-2 py-1 font-medium text-base-content/40">SWS</th>
                        <th class="text-left px-2 py-1 font-medium text-base-content/40">Language</th>
                    </tr>
                </thead>
                <tbody>
                    {components.iter().map(|comp| {
                        let comp = comp.clone();
                        view! {
                            <tr class="bg-base-200/50">
                                <td class="px-2 py-1.5">
                                    <div class="badge badge-soft badge-accent badge-sm">
                                        {comp.component_type}
                                    </div>
                                </td>
                                <td class="px-2 py-1.5">
                                    {if let Some(name) = comp.name {
                                        view! {
                                            <span class="font-medium">{name}</span>
                                        }.into_any()
                                    } else {
                                        view! { <span></span> }.into_any()
                                    }}
                                </td>
                                <td class="px-2 py-1.5 text-base-content/60">{comp.number}</td>
                                <td class="px-2 py-1.5 text-base-content/60">{comp.rotation}</td>
                                <td class="px-2 py-1.5 text-base-content/60">{comp.sws} " SWS"</td>
                                <td class="px-2 py-1.5 text-base-content/60">{comp.language}</td>
                            </tr>
                        }
                    }).collect_view()}
                </tbody>
            </table>
        </div>
    }
    .into_any()
}
