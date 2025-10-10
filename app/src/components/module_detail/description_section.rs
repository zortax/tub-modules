use leptos::prelude::*;

#[component]
pub fn DescriptionSection(
    learning_result: Option<String>,
    content: Option<String>,
    teaching_information: Option<String>,
) -> impl IntoView {
    let has_learning = learning_result.is_some();
    let has_content = content.is_some();
    let has_teaching = teaching_information.is_some();
    let has_any = has_learning || has_content || has_teaching;

    if !has_any {
        return view! { <></> }.into_any();
    }

    view! {
        <div class="card bg-base-100 shadow-sm">
            <div class="card-body p-5 gap-5">
                {if let Some(learning) = learning_result {
                    view! {
                        <div>
                            <h3 class="font-semibold text-base mb-3 text-primary">Learning Objectives</h3>
                            <div class="prose prose-sm max-w-none text-base-content/80">
                                <p class="whitespace-pre-wrap leading-relaxed">{learning}</p>
                            </div>
                        </div>
                    }.into_any()
                } else {
                    view! { <></> }.into_any()
                }}

                {if has_learning && has_content {
                    view! { <div class="divider my-1 opacity-50"></div> }.into_any()
                } else {
                    view! { <></> }.into_any()
                }}

                {if let Some(desc) = content {
                    view! {
                        <div>
                            <h3 class="font-semibold text-base mb-3 text-primary">Module Content</h3>
                            <div class="prose prose-sm max-w-none text-base-content/80">
                                <p class="whitespace-pre-wrap leading-relaxed">{desc}</p>
                            </div>
                        </div>
                    }.into_any()
                } else {
                    view! { <></> }.into_any()
                }}

                {if (has_learning || has_content) && has_teaching {
                    view! { <div class="divider my-1 opacity-50"></div> }.into_any()
                } else {
                    view! { <></> }.into_any()
                }}

                {if let Some(teaching) = teaching_information {
                    view! {
                        <div>
                            <h3 class="font-semibold text-base mb-3 text-primary">Teaching Information</h3>
                            <div class="prose prose-sm max-w-none text-base-content/80">
                                <p class="whitespace-pre-wrap leading-relaxed">{teaching}</p>
                            </div>
                        </div>
                    }.into_any()
                } else {
                    view! { <></> }.into_any()
                }}
            </div>
        </div>
    }
    .into_any()
}
