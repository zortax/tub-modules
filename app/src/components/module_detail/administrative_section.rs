use leptos::prelude::*;

#[component]
pub fn AdministrativeSection(
    max_attendees: Option<i32>,
    duration: Option<String>,
    additional_info: Option<String>,
) -> impl IntoView {
    let has_content = max_attendees.is_some() || duration.is_some() || additional_info.is_some();

    if !has_content {
        return view! { <></> }.into_any();
    }

    view! {
        <div class="card bg-base-100 shadow-sm">
            <div class="card-body p-5">
                <h2 class="card-title text-lg text-primary mb-1">Administrative Information</h2>
                <div class="divider my-0 opacity-50"></div>
                <div class="grid grid-cols-1 md:grid-cols-2 gap-x-6 gap-y-3 text-sm mt-3">
                    {if let Some(attendees) = max_attendees {
                        view! {
                            <div class="flex flex-col gap-0.5">
                                <span class="text-xs text-base-content/50 uppercase tracking-wide">Max Attendees</span>
                                <span class="font-medium text-base-content">{attendees}</span>
                            </div>
                        }.into_any()
                    } else {
                        view! { <></> }.into_any()
                    }}
                    {if let Some(dur) = duration {
                        view! {
                            <div class="flex flex-col gap-0.5">
                                <span class="text-xs text-base-content/50 uppercase tracking-wide">Duration</span>
                                <span class="font-medium text-base-content">{dur}</span>
                            </div>
                        }.into_any()
                    } else {
                        view! { <></> }.into_any()
                    }}
                </div>
                {if let Some(info) = additional_info {
                    view! {
                        <div>
                            <div class="divider my-2 opacity-50"></div>
                            <h3 class="font-semibold text-base mb-3 text-primary">Additional Information</h3>
                            <div class="prose prose-sm max-w-none text-base-content/80">
                                <p class="whitespace-pre-wrap leading-relaxed">{info}</p>
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
