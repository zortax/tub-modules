use leptos::prelude::*;

#[component]
pub fn RequirementsSection(
    requirements: Option<String>,
    registration: Option<String>,
) -> impl IntoView {
    let has_requirements = requirements.is_some();
    let has_registration = registration.is_some();
    let has_content = has_requirements || has_registration;

    if !has_content {
        return view! { <></> }.into_any();
    }

    view! {
        <div class="card bg-base-100 shadow-sm">
            <div class="card-body p-5 gap-5">
                {if let Some(reqs) = requirements {
                    view! {
                        <div>
                            <h3 class="font-semibold text-base mb-3 text-primary">Requirements</h3>
                            <div class="prose prose-sm max-w-none text-base-content/80">
                                <p class="whitespace-pre-wrap leading-relaxed">{reqs}</p>
                            </div>
                        </div>
                    }.into_any()
                } else {
                    view! { <></> }.into_any()
                }}

                {if has_requirements && has_registration {
                    view! { <div class="divider my-1 opacity-50"></div> }.into_any()
                } else {
                    view! { <></> }.into_any()
                }}

                {if let Some(reg) = registration {
                    view! {
                        <div>
                            <h3 class="font-semibold text-base mb-3 text-primary">Registration</h3>
                            <div class="prose prose-sm max-w-none text-base-content/80">
                                <p class="whitespace-pre-wrap leading-relaxed">{reg}</p>
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
