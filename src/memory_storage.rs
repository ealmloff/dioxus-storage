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

use crate::storage::{serde_to_string, try_serde_from_string};

static mut CACHED: Lazy<Storage> = Lazy::new(|| {
    // try read from temp storage
    Storage::default()
});

#[derive(Default)]
struct Storage {
    data: RwLock<HashMap<String, Vec<u8>>>,
}

impl Drop for Storage {
    fn drop(&mut self) {
        let serialized = serde_to_string(&self.data);
    }
}

#[derive(Clone)]
pub struct StorageEntry<T: Serialize + for<'a> Deserialize<'a>> {
    data: T,
    created: std::time::Instant,
}

impl<T: Display + Serialize + for<'a> Deserialize<'a>> StorageEntry<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.data.fmt(f)
    }
}

pub fn temp<T: Serialize + for<'a> Deserialize<'a>>(
    key: &'static str,
    timeout: Duration,
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
