use leptos::prelude::*;

#[component]
pub fn MetadataSection(
    faculty: String,
    institute: String,
    fachgebiet: String,
    responsible_person: String,
    examination_board: String,
    valid_since: Option<String>,
    valid_until: Option<String>,
) -> impl IntoView {
    view! {
        <div class="card bg-base-100 shadow-sm">
            <div class="card-body p-5">
                <h2 class="card-title text-lg text-primary mb-1">Organization & Validity</h2>
                <div class="divider my-0 opacity-50"></div>
                <div class="grid grid-cols-1 md:grid-cols-2 gap-x-6 gap-y-3 text-sm mt-3">
                    {if !faculty.is_empty() {
                        view! {
                            <div class="flex flex-col gap-0.5">
                                <span class="text-xs text-base-content/50 uppercase tracking-wide">Faculty</span>
                                <span class="font-medium text-base-content">{faculty}</span>
                            </div>
                        }.into_any()
                    } else {
                        view! { <></> }.into_any()
                    }}
                    {if !institute.is_empty() {
                        view! {
                            <div class="flex flex-col gap-0.5">
                                <span class="text-xs text-base-content/50 uppercase tracking-wide">Institute</span>
                                <span class="font-medium text-base-content">{institute}</span>
                            </div>
                        }.into_any()
                    } else {
                        view! { <></> }.into_any()
                    }}
                    {if !fachgebiet.is_empty() {
                        view! {
                            <div class="flex flex-col gap-0.5">
                                <span class="text-xs text-base-content/50 uppercase tracking-wide">Fachgebiet</span>
                                <span class="font-medium text-base-content">{fachgebiet}</span>
                            </div>
                        }.into_any()
                    } else {
                        view! { <></> }.into_any()
                    }}
                    {if !responsible_person.is_empty() {
                        view! {
                            <div class="flex flex-col gap-0.5">
                                <span class="text-xs text-base-content/50 uppercase tracking-wide">Responsible Person</span>
                                <span class="font-medium text-base-content">{responsible_person}</span>
                            </div>
                        }.into_any()
                    } else {
                        view! { <></> }.into_any()
                    }}
                    {if !examination_board.is_empty() {
                        view! {
                            <div class="flex flex-col gap-0.5">
                                <span class="text-xs text-base-content/50 uppercase tracking-wide">Examination Board</span>
                                <span class="font-medium text-base-content">{examination_board}</span>
                            </div>
                        }.into_any()
                    } else {
                        view! { <></> }.into_any()
                    }}
                    {if let Some(since) = valid_since {
                        view! {
                            <div class="flex flex-col gap-0.5">
                                <span class="text-xs text-base-content/50 uppercase tracking-wide">Valid Since</span>
                                <span class="font-medium text-base-content">{since}</span>
                            </div>
                        }.into_any()
                    } else {
                        view! { <></> }.into_any()
                    }}
                    {if let Some(until) = valid_until {
                        view! {
                            <div class="flex flex-col gap-0.5">
                                <span class="text-xs text-base-content/50 uppercase tracking-wide">Valid Until</span>
                                <span class="font-medium text-base-content">{until}</span>
                            </div>
                        }.into_any()
                    } else {
                        view! { <></> }.into_any()
                    }}
                </div>
            </div>
        </div>
    }
}
