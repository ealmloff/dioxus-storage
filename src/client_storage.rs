#![allow(unused)]
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::{
    fmt::Display,
    ops::{Deref, DerefMut},
};
use web_sys::{window, Storage};

use crate::storage::{
    serde_from_string, serde_to_string, storage_entry, try_serde_from_string,
    use_synced_storage_entry, StorageBacking, StorageEntry,
};

fn local_storage() -> Storage {
    window().unwrap().local_storage().unwrap().unwrap()
}

fn set<T: Serialize>(key: String, value: &T) {
    #[cfg(target_arch = "wasm32")]
    {
        let as_str = serde_to_string(value);
        local_storage().set_item(&key, &as_str).unwrap();
    }
}

fn get<T: for<'a> Deserialize<'a>>(key: &str) -> Option<T> {
    #[cfg(target_arch = "wasm32")]
    {
        let s: String = local_storage().get_item(key).ok()??;
        return try_serde_from_string(&s);
    }
    None
}

pub struct ClientStorage;

impl StorageBacking for ClientStorage {
    type Key = String;

    fn set<T: Serialize>(key: String, value: &T) {
        set(key, value);
    }

    fn get<T: for<'a> Deserialize<'a>>(key: &String) -> Option<T> {
        get(key)
    }
}

#[allow(clippy::needless_return)]
pub fn use_persistant<'a, T: Serialize + for<'b> Deserialize<'b> + Default + 'static>(
    cx: &'a ScopeState,
    key: impl ToString,
    init: impl FnOnce() -> T,
) -> &'a UseRef<StorageEntry<ClientStorage, T>> {
    let mut init = Some(init);
    let state = use_ref(cx, || {
        StorageEntry::<ClientStorage, T>::new(key.to_string(), init.take().unwrap()())
    });
    #[cfg(target_arch = "wasm32")]
    {
        if cx.generation() == 0 {
            cx.needs_update();
        }
        if cx.generation() == 1 {
            state.set(StorageEntry::new(
                key.to_string(),
                storage_entry::<ClientStorage, T>(key.to_string(), init.take().unwrap()),
            ));
        }
    }
    state
}
