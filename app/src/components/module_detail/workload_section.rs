use crate::models::WorkloadInfo;
use leptos::prelude::*;

#[component]
pub fn WorkloadSection(workload: Vec<WorkloadInfo>) -> impl IntoView {
    if workload.is_empty() {
        return view! { <></> }.into_any();
    }

    let total_hours: f64 = workload.iter().map(|w| w.hours).sum();

    view! {
        <div class="card bg-base-100 shadow-sm">
            <div class="card-body p-5">
                <h2 class="card-title text-lg text-primary mb-1">Workload Distribution</h2>
                <div class="divider my-0 opacity-50"></div>
                <div class="mt-3 overflow-hidden rounded-lg">
                    <table class="w-full text-sm" style="border-spacing: 0 1px; border-collapse: separate;">
                        <thead>
                            <tr class="bg-base-200/30">
                                <th class="text-left px-2 py-1 font-medium text-base-content/40">Description</th>
                                <th class="text-right px-2 py-1 font-medium text-base-content/40">Hours</th>
                            </tr>
                        </thead>
                        <tbody>
                            {workload.iter().map(|w| {
                                let w = w.clone();
                                view! {
                                    <tr class="bg-base-200/50">
                                        <td class="px-2 py-1.5">{w.description}</td>
                                        <td class="px-2 py-1.5 text-right text-base-content/60">{w.hours}</td>
                                    </tr>
                                }
                            }).collect_view()}
                            <tr class="bg-primary/10">
                                <td class="px-2 py-1.5 font-semibold">Total</td>
                                <td class="px-2 py-1.5 text-right font-semibold">{total_hours}</td>
                            </tr>
                        </tbody>
                    </table>
                </div>
            </div>
        </div>
    }
    .into_any()
}
