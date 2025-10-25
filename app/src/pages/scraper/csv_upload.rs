use leptos::prelude::*;
use leptos::task::spawn_local;
use crate::api::{upload_csv, CsvInfoDto};

#[component]
pub fn CsvUpload(
    auth_key: Signal<String>,
    on_uploaded: impl Fn(CsvInfoDto) + 'static + Copy,
) -> impl IntoView {
    let (uploading, set_uploading) = signal(false);
    let (error, set_error) = signal(None::<String>);
    let (dragging, set_dragging) = signal(false);
    let file_input_ref = NodeRef::<leptos::html::Input>::new();

    #[allow(unused_variables)]
    let handle_file = move |file_name: String, content: String| {
        let auth = auth_key.get_untracked();

        spawn_local(async move {
            set_uploading.set(true);
            set_error.set(None);

            match upload_csv(content, file_name, auth).await {
                Ok(response) => {
                    if response.success {
                        if let Some(info) = response.info {
                            on_uploaded(info);
                        }
                        set_error.set(None);
                    } else {
                        set_error.set(Some(response.message));
                    }
                }
                Err(e) => {
                    set_error.set(Some(format!("Upload failed: {}", e)));
                }
            }

            set_uploading.set(false);
        });
    };

    let on_file_selected = move |_| {
        if let Some(input) = file_input_ref.get() {
            if let Some(files) = input.files() {
                #[allow(unused_variables)]
                if let Some(file) = files.get(0) {
                    #[cfg(target_arch = "wasm32")]
                    {
                        let file_name = file.name();
                        let file_clone = file.clone();
                        let handle = handle_file;

                        spawn_local(async move {
                            use wasm_bindgen_futures::JsFuture;

                            let promise = file_clone.text();
                            match JsFuture::from(promise).await {
                                Ok(content) => {
                                    if let Some(content_str) = content.as_string() {
                                        handle(file_name, content_str);
                                    }
                                }
                                Err(_) => {
                                    set_error.set(Some("Failed to read file".to_string()));
                                }
                            }
                        });
                    }
                }
            }
        }
    };

    let on_drop = move |ev: web_sys::DragEvent| {
        ev.prevent_default();
        set_dragging.set(false);

        #[cfg(target_arch = "wasm32")]
        {
            if let Some(data_transfer) = ev.data_transfer() {
                if let Some(files) = data_transfer.files() {
                    if let Some(file) = files.get(0) {
                        use wasm_bindgen_futures::JsFuture;

                        let file_name = file.name();
                        let file_clone = file.clone();
                        let handle = handle_file;

                        spawn_local(async move {
                            let promise = file_clone.text();
                            match JsFuture::from(promise).await {
                                Ok(content) => {
                                    if let Some(content_str) = content.as_string() {
                                        handle(file_name, content_str);
                                    }
                                }
                                Err(_) => {
                                    set_error.set(Some("Failed to read file".to_string()));
                                }
                            }
                        });
                    }
                }
            }
        }
    };

    let on_drag_over = move |ev: web_sys::DragEvent| {
        ev.prevent_default();
        set_dragging.set(true);
    };

    let on_drag_leave = move |_| {
        set_dragging.set(false);
    };

    view! {
        <div class="card bg-base-100 shadow-xl">
            <div class="card-body">
                <h2 class="card-title">
                    <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12" />
                    </svg>
                    "Upload CSV File"
                </h2>

                <div
                    class=move || {
                        format!(
                            "border-2 border-dashed rounded-lg p-12 text-center transition-colors {}",
                            if dragging.get() {
                                "border-primary bg-primary/10"
                            } else {
                                "border-base-300 hover:border-primary/50"
                            }
                        )
                    }
                    on:drop=on_drop
                    on:dragover=on_drag_over
                    on:dragleave=on_drag_leave
                >
                    <div class="space-y-4">
                        <svg xmlns="http://www.w3.org/2000/svg" class="h-16 w-16 mx-auto text-base-content/40" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
                        </svg>

                        <div class="text-lg">
                            "Drag and drop your CSV file here"
                        </div>

                        <div class="text-sm text-base-content/60">
                            "or"
                        </div>

                        <input
                            type="file"
                            accept=".csv"
                            class="hidden"
                            node_ref=file_input_ref
                            on:change=on_file_selected
                        />

                        <button
                            class="btn btn-soft btn-primary"
                            on:click=move |_| {
                                if let Some(input) = file_input_ref.get() {
                                    input.click();
                                }
                            }
                            disabled=move || uploading.get()
                        >
                            {move || if uploading.get() {
                                "Uploading..."
                            } else {
                                "Choose File"
                            }}
                        </button>

                        <div class="text-xs text-base-content/40 mt-2">
                            "Expected format: MOSES module export CSV"
                        </div>
                    </div>
                </div>

                {move || error.get().map(|err| view! {
                    <div class="alert alert-error mt-4">
                        <svg xmlns="http://www.w3.org/2000/svg" class="stroke-current shrink-0 h-6 w-6" fill="none" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2m7-2a9 9 0 11-18 0 9 9 0 0118 0z" />
                        </svg>
                        <span>{err}</span>
                    </div>
                })}
            </div>
        </div>
    }
}
