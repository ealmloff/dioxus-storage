#![allow(unused)]
use dioxus::prelude::*;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::RwLock;
use std::time::Duration;
use std::{
    fmt::Display,
    ops::{Deref, DerefMut},
};
use web_sys::window;

use crate::storage::{serde_to_string, try_serde_from_string, StorageBacking};

static mut CACHED: Lazy<Storage> = Lazy::new(|| {
    // try read from temp storage
    Storage::default()
});

pub struct MemoryStorage;

impl<K: Serialize + DeserializeOwned> StorageBacking for MemoryStorage {
    type Key = K;

    fn get<T: DeserializeOwned>(key: &Self::Key) -> Option<T> {}

    fn set<T: Serialize>(key: Self::Key, value: &T) {
        todo!()
    }
}

pub fn use_temp<T: Serialize + DeserializeOwned>(
    cx: &ScopeState,
    key: &'static str,
    init: impl FnOnce() -> T,
) -> T {
    #[cfg(target_arch = "wasm32")]
    return init();
    #[cfg(not(target_arch = "wasm32"))]
    {
        let mut entry = StorageEntry::new(key, init());
        if entry.is_expired(timeout) {
            entry.data = init();
            entry.save();
        }
        entry.data
    }
}
