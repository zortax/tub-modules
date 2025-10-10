use crate::models::StudyProgramInfo;
use leptos::prelude::*;

#[component]
pub fn ProgramsSection(programs: Vec<StudyProgramInfo>) -> impl IntoView {
    if programs.is_empty() {
        return view! { <></> }.into_any();
    }

    view! {
        <div class="card bg-base-100 shadow-sm">
            <div class="card-body p-5">
                <h2 class="card-title text-lg text-primary mb-1">Study Programs</h2>
                <div class="divider my-0 opacity-50"></div>
                <div class="grid grid-cols-1 md:grid-cols-2 gap-3 mt-3">
                    {programs.into_iter().map(|prog| {
                        view! {
                            <div class="border border-base-300/50 bg-base-200/20 rounded-lg p-3 hover:bg-base-200/40 transition-colors">
                                <div class="font-semibold text-sm text-base-content mb-1">{prog.program_name}</div>
                                <div class="text-xs text-base-content/60 mb-2">{prog.stupo_name}</div>
                                <div class="flex items-center gap-1.5 text-xs text-base-content/50">
                                    <span class="badge badge-soft badge-xs">{prog.first_usage}</span>
                                    <span>"→"</span>
                                    <span class="badge badge-soft badge-xs">{prog.last_usage}</span>
                                </div>
                            </div>
                        }
                    }).collect_view()}
                </div>
            </div>
        </div>
    }
    .into_any()
}
