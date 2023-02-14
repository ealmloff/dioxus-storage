#![allow(unused)]
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::{
    fmt::Display,
    ops::{Deref, DerefMut},
};
use web_sys::{window, Storage};

use crate::storage::{serde_to_string, try_serde_from_string};

#[derive(Clone, Default, Debug)]
struct LocalStorage;

fn local_storage() -> Storage {
    window().unwrap().local_storage().unwrap().unwrap()
}

fn set(key: String, value: String) {
    local_storage().set_item(&key, &value).unwrap();
}

fn get(key: String) -> Option<String> {
    local_storage().get_item(&key).unwrap()
}

#[derive(Clone, Default)]
pub struct Persistant<T: Serialize + for<'a> Deserialize<'a>> {
    key: &'static str,
    data: T,
}

impl<T: Display + Serialize + for<'a> Deserialize<'a>> Display for Persistant<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.data.fmt(f)
    }
}

impl<T: Debug + Serialize + for<'a> Deserialize<'a>> Debug for Persistant<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.data.fmt(f)
    }
}

impl<T: Serialize + for<'a> Deserialize<'a>> Persistant<T> {
    pub fn new(key: &'static str, data: T) -> Self {
        Self { key, data }
    }

    fn save(&self) {
        #[cfg(target_arch = "wasm32")]
        {
            let data = serde_to_string(&self.data);
            set(self.key.to_string(), data);
        }
    }

    pub fn write(&mut self) -> PersistantMut<'_, T> {
        PersistantMut { persistant: self }
    }

    pub fn with_mut(&mut self, f: impl FnOnce(&mut T)) {
        f(&mut self.data);
        self.save();
    }
}

impl<T: Serialize + for<'a> Deserialize<'a>> Deref for Persistant<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

pub struct PersistantMut<'a, T: Serialize + for<'b> Deserialize<'b>> {
    persistant: &'a mut Persistant<T>,
}

impl<'a, T: Serialize + for<'b> Deserialize<'b>> Deref for PersistantMut<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.persistant.data
    }
}

impl<'a, T: Serialize + for<'b> Deserialize<'b>> DerefMut for PersistantMut<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.persistant.data
    }
}

impl<'a, T: Serialize + for<'b> Deserialize<'b>> Drop for PersistantMut<'a, T> {
    fn drop(&mut self) {
        self.persistant.save();
    }
}

pub fn persistant<T: Serialize + for<'a> Deserialize<'a>>(
    key: &'static str,
    init: impl FnOnce() -> T,
) -> Persistant<T> {
    #[cfg(target_arch = "wasm32")]
    let data = match get(key.to_string()) {
        Some(data) => try_serde_from_string(&data).unwrap_or_else(init),
        None => init(),
    };
    #[cfg(not(target_arch = "wasm32"))]
    let data = init();

    Persistant::new(key, data)
}

pub fn use_persistant<'a, T: Serialize + for<'b> Deserialize<'b> + Default + 'static>(
    cx: &'a ScopeState,
    key: &'static str,
    init: impl FnOnce() -> T,
) -> &'a UseRef<Persistant<T>> {
    let mut init = Some(init);
    let count = use_ref(cx, || Persistant::new("", init.take().unwrap()()));
    if cx.generation() == 0 {
        cx.needs_update();
    }
    if cx.generation() == 1 {
        count.set(persistant("count", init.take().unwrap()));
    }
    count
}
