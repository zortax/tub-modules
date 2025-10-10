use crate::models::ContactInfo;
use leptos::prelude::*;

#[component]
pub fn ContactSection(contact: Option<ContactInfo>) -> impl IntoView {
    if contact.is_none() {
        return view! { <></> }.into_any();
    }

    let contact = contact.unwrap();
    let has_content = contact.secretariat.is_some()
        || contact.contact_person.is_some()
        || contact.email.is_some()
        || contact.website.is_some();

    if !has_content {
        return view! { <></> }.into_any();
    }

    view! {
        <div class="card bg-base-100 shadow-sm">
            <div class="card-body p-5">
                <h2 class="card-title text-lg text-primary mb-1">Contact Information</h2>
                <div class="divider my-0 opacity-50"></div>
                <div class="grid grid-cols-1 md:grid-cols-2 gap-x-6 gap-y-3 text-sm mt-3">
                    {if let Some(secretariat) = contact.secretariat {
                        view! {
                            <div class="flex flex-col gap-0.5">
                                <span class="text-xs text-base-content/50 uppercase tracking-wide">Secretariat</span>
                                <span class="font-medium text-base-content">{secretariat}</span>
                            </div>
                        }.into_any()
                    } else {
                        view! { <></> }.into_any()
                    }}
                    {if let Some(person) = contact.contact_person {
                        view! {
                            <div class="flex flex-col gap-0.5">
                                <span class="text-xs text-base-content/50 uppercase tracking-wide">Contact Person</span>
                                <span class="font-medium text-base-content">{person}</span>
                            </div>
                        }.into_any()
                    } else {
                        view! { <></> }.into_any()
                    }}
                    {if let Some(ref email) = contact.email {
                        let email_clone = email.clone();
                        let mailto = format!("mailto:{}", email);
                        view! {
                            <div class="flex flex-col gap-0.5">
                                <span class="text-xs text-base-content/50 uppercase tracking-wide">Email</span>
                                <a href=mailto class="link link-primary font-medium hover:underline">{email_clone}</a>
                            </div>
                        }.into_any()
                    } else {
                        view! { <></> }.into_any()
                    }}
                    {if let Some(ref website) = contact.website {
                        let website_clone = website.clone();
                        let url = website.clone();
                        view! {
                            <div class="flex flex-col gap-0.5">
                                <span class="text-xs text-base-content/50 uppercase tracking-wide">Website</span>
                                <a href=url target="_blank" rel="noopener noreferrer" class="link link-primary font-medium hover:underline">{website_clone}</a>
                            </div>
                        }.into_any()
                    } else {
                        view! { <></> }.into_any()
                    }}
                </div>
            </div>
        </div>
    }
    .into_any()
}
