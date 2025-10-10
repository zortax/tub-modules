use crate::models::SearchFilters;
use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[cfg(target_arch = "wasm32")]
use codee::string::JsonSerdeCodec;
#[cfg(target_arch = "wasm32")]
use leptos_use::storage::use_local_storage;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct StarredModules {
    pub modules: HashSet<(i32, i32)>, // (id, version)
}

pub fn use_starred_modules() -> (
    Signal<StarredModules>,
    WriteSignal<StarredModules>,
) {
    #[cfg(target_arch = "wasm32")]
    {
        let (starred, set_starred, _remove) = use_local_storage::<StarredModules, JsonSerdeCodec>("starred_modules");
        (starred, set_starred)
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        // SSR fallback - just return dummy signals
        let starred = RwSignal::new(StarredModules::default());
        (starred.read_only().into(), starred.write_only())
    }
}

pub fn is_starred(starred: &StarredModules, id: i32, version: i32) -> bool {
    starred.modules.contains(&(id, version))
}

pub fn toggle_starred(
    starred: &StarredModules,
    id: i32,
    version: i32,
) -> StarredModules {
    let mut new_starred = starred.clone();
    let key = (id, version);

    if new_starred.modules.contains(&key) {
        new_starred.modules.remove(&key);
    } else {
        new_starred.modules.insert(key);
    }

    new_starred
}

pub fn use_search_filters() -> RwSignal<SearchFilters> {
    #[cfg(target_arch = "wasm32")]
    {
        let (stored_filters, set_stored_filters, _remove) =
            use_local_storage::<SearchFilters, JsonSerdeCodec>("search_filters");

        // Create an RwSignal initialized with stored value
        let filters = RwSignal::new(stored_filters.get_untracked());

        // Sync changes back to local storage
        Effect::new(move || {
            let current = filters.get();
            set_stored_filters.set(current);
        });

        filters
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        // SSR fallback - just return a regular signal
        RwSignal::new(SearchFilters::default())
    }
}
