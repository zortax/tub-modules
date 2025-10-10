use crate::components::shared::ExamTable;
use crate::models::ExamInfo;
use leptos::prelude::*;

#[component]
pub fn ExamsSection(exams: Vec<ExamInfo>) -> impl IntoView {
    if exams.is_empty() {
        return view! { <></> }.into_any();
    }

    view! {
        <div class="card bg-base-100 shadow-sm">
            <div class="card-body p-5 gap-5">
                <h2 class="card-title text-lg text-primary">Exams & Assessment</h2>
                <div class="divider my-0 opacity-50"></div>
                {exams.into_iter().enumerate().map(|(idx, exam)| {
                    view! {
                        <div>
                            {if idx > 0 {
                                view! { <div class="divider my-3 opacity-50"></div> }.into_any()
                            } else {
                                view! { <></> }.into_any()
                            }}
                            <div class="flex items-center gap-2 mb-3">
                                <h3 class="font-semibold text-base text-base-content">{exam.exam_type.clone()}</h3>
                                <div class="badge badge-soft badge-sm">
                                    {if exam.graded { "✓ Graded" } else { "Ungraded" }}
                                </div>
                            </div>
                            {if let Some(desc) = exam.description.clone() {
                                view! {
                                    <div class="prose prose-sm max-w-none mb-3">
                                        <p class="text-sm text-base-content/70 leading-relaxed">{desc}</p>
                                    </div>
                                }.into_any()
                            } else {
                                view! { <></> }.into_any()
                            }}
                            <ExamTable components=exam.components />
                        </div>
                    }
                }).collect_view()}
            </div>
        </div>
    }
    .into_any()
}
