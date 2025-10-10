use crate::models::ExamComponentInfo;
use leptos::prelude::*;

#[component]
pub fn ExamTable(components: Vec<ExamComponentInfo>) -> impl IntoView {
    if components.is_empty() {
        return view! { <></> }.into_any();
    }

    view! {
        <div class="overflow-hidden rounded-lg">
            <table class="w-full text-sm" style="border-spacing: 0 1px; border-collapse: separate;">
                <thead>
                    <tr class="bg-base-200/30">
                        <th class="text-left px-2 py-1 font-medium text-base-content/40">Name</th>
                        <th class="text-left px-2 py-1 font-medium text-base-content/40">Category</th>
                        <th class="text-left px-2 py-1 font-medium text-base-content/40">Points</th>
                        <th class="text-left px-2 py-1 font-medium text-base-content/40">Scope</th>
                    </tr>
                </thead>
                <tbody>
                    {components.iter().map(|comp| {
                        let comp = comp.clone();
                        view! {
                            <tr class="bg-base-200/50">
                                <td class="px-2 py-1.5 font-medium">{comp.name}</td>
                                <td class="px-2 py-1.5">
                                    <div class="badge badge-soft badge-accent badge-sm">
                                        {comp.category}
                                    </div>
                                </td>
                                <td class="px-2 py-1.5 text-base-content/60">{comp.points}</td>
                                <td class="px-2 py-1.5 text-base-content/60">
                                    {if let Some(scope) = comp.scope {
                                        view! { <span>{scope}</span> }.into_any()
                                    } else {
                                        view! { <span class="text-base-content/30">"-"</span> }.into_any()
                                    }}
                                </td>
                            </tr>
                        }
                    }).collect_view()}
                </tbody>
            </table>
        </div>
    }
    .into_any()
}
