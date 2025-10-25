pub mod api;
pub mod app;
pub mod components;
pub mod models;
pub mod pages;
pub mod starred;
pub mod scraper_types;

#[cfg(feature = "ssr")]
pub mod scraper_state;

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use app::*;
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(App);
}
