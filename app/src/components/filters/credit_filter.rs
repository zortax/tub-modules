use crate::models::{FilterOptions, SearchFilters};
use leptos::prelude::*;
use leptos::ev;
use wasm_bindgen::JsCast;

#[component]
pub fn CreditFilter(
    filters: RwSignal<SearchFilters>,
    filter_options: Signal<Option<FilterOptions>>,
) -> impl IntoView {
    let min_value = RwSignal::new(0_i32);
    let max_value = RwSignal::new(100_i32);
    let dragging = RwSignal::new(None::<DragTarget>);
    let pending_update = RwSignal::new(false);

    #[derive(Clone, Copy, PartialEq)]
    enum DragTarget {
        Min,
        Max,
    }

    // Sync slider values with filter state and database range
    Effect::new(move || {
        if let Some(opts) = filter_options.get() {
            let (db_min, db_max) = opts.credit_range;
            let current_filters = filters.get();

            // Check if filters were reset (both None)
            if current_filters.min_credits.is_none() && current_filters.max_credits.is_none() {
                min_value.set(db_min);
                max_value.set(db_max);

                let mut new_filters = current_filters;
                new_filters.min_credits = Some(db_min);
                new_filters.max_credits = Some(db_max);
                filters.set(new_filters);
            } else {
                if let Some(min) = current_filters.min_credits {
                    min_value.set(min);
                }
                if let Some(max) = current_filters.max_credits {
                    max_value.set(max);
                }
            }
        }
    });

    view! {
        <div class="form-control">
            <label class="label pb-1">
                <span class="label-text font-semibold text-base flex items-center gap-2">
                    <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 20 20" fill="currentColor">
                        <path fill-rule="evenodd" d="M6 2a1 1 0 00-1 1v1H4a2 2 0 00-2 2v10a2 2 0 002 2h12a2 2 0 002-2V6a2 2 0 00-2-2h-1V3a1 1 0 10-2 0v1H7V3a1 1 0 00-1-1zm0 5a1 1 0 000 2h8a1 1 0 100-2H6z" clip-rule="evenodd" />
                    </svg>
                    "Credit Points"
                </span>
            </label>
            {move || {
                filter_options.get().map(|opts| {
                    let (db_min, db_max) = opts.credit_range;
                    let range = (db_max - db_min) as f32;

                    let handle_mouse_move = move |ev: ev::MouseEvent| {
                        if let Some(target) = dragging.get() {
                            if let Some(elem) = ev.current_target() {
                                let elem = elem.unchecked_into::<web_sys::HtmlElement>();
                                let rect = elem.get_bounding_client_rect();
                                let x = (ev.client_x() as f64 - rect.left()) / rect.width();
                                let x = x.max(0.0).min(1.0);
                                let value = db_min + (x * range as f64) as i32;

                                match target {
                                    DragTarget::Min => {
                                        let capped = value.min(max_value.get());
                                        min_value.set(capped);
                                        pending_update.set(true);
                                    }
                                    DragTarget::Max => {
                                        let capped = value.max(min_value.get());
                                        max_value.set(capped);
                                        pending_update.set(true);
                                    }
                                }
                            }
                        }
                    };

                    let handle_mouse_up = move |_| {
                        if dragging.get().is_some() {
                            dragging.set(None);

                            // Apply filter update after drag ends
                            if pending_update.get() {
                                filters.update(|f| {
                                    f.min_credits = Some(min_value.get());
                                    f.max_credits = Some(max_value.get());
                                });
                                pending_update.set(false);
                            }
                        }
                    };

                    view! {
                        <div class="mt-2">
                            <div class="flex justify-between text-xs text-base-content/60 mb-2">
                                <span>{move || min_value.get()} " CP"</span>
                                <span>{move || max_value.get()} " CP"</span>
                            </div>

                            <div
                                class="relative h-8 select-none cursor-pointer"
                                on:mousemove=handle_mouse_move
                                on:mouseup=handle_mouse_up
                                on:mouseleave=handle_mouse_up
                            >
                                // Background track
                                <div class="absolute top-1/2 -translate-y-1/2 w-full h-1.5 bg-base-300 rounded-full"></div>

                                // Active range
                                <div
                                    class="absolute top-1/2 -translate-y-1/2 h-1.5 bg-primary rounded-full"
                                    style:left=move || {
                                        let pos = ((min_value.get() - db_min) as f32 / range * 100.0).max(0.0).min(100.0);
                                        format!("{}%", pos)
                                    }
                                    style:right=move || {
                                        let pos = ((db_max - max_value.get()) as f32 / range * 100.0).max(0.0).min(100.0);
                                        format!("{}%", pos)
                                    }
                                ></div>

                                // Min thumb
                                <div
                                    class="absolute top-1/2 -translate-y-1/2 -translate-x-1/2 w-4 h-4 bg-primary rounded-full border-2 border-base-100 cursor-grab active:cursor-grabbing hover:scale-110 transition-transform shadow-md"
                                    style:left=move || {
                                        let pos = ((min_value.get() - db_min) as f32 / range * 100.0).max(0.0).min(100.0);
                                        format!("{}%", pos)
                                    }
                                    on:mousedown=move |ev| {
                                        ev.prevent_default();
                                        dragging.set(Some(DragTarget::Min));
                                    }
                                ></div>

                                // Max thumb
                                <div
                                    class="absolute top-1/2 -translate-y-1/2 -translate-x-1/2 w-4 h-4 bg-primary rounded-full border-2 border-base-100 cursor-grab active:cursor-grabbing hover:scale-110 transition-transform shadow-md"
                                    style:left=move || {
                                        let pos = ((max_value.get() - db_min) as f32 / range * 100.0).max(0.0).min(100.0);
                                        format!("{}%", pos)
                                    }
                                    on:mousedown=move |ev| {
                                        ev.prevent_default();
                                        dragging.set(Some(DragTarget::Max));
                                    }
                                ></div>
                            </div>

                            <div class="flex justify-between text-xs text-base-content/40 mt-1">
                                <span>{db_min}</span>
                                <span>{db_max}</span>
                            </div>
                        </div>
                    }
                })
            }}
        </div>
    }
}
