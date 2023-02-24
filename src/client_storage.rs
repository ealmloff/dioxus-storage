#![allow(unused)]
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use std::cell::{Ref, RefMut};
use std::fmt::Debug;
use std::{
    fmt::Display,
    ops::{Deref, DerefMut},
};
use web_sys::{window, Storage};

use crate::storage::{
    serde_from_string, serde_to_string, storage_entry, try_serde_from_string,
    use_synced_storage_entry, StorageBacking, StorageEntry, StorageEntryMut,
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
) -> &'a UsePersistant<T> {
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
    cx.use_hook(|| UsePersistant {
        inner: state.clone(),
    })
}

pub struct StorageRef<'a, T: Serialize + for<'b> Deserialize<'b> + Default + 'static> {
    inner: Ref<'a, StorageEntry<ClientStorage, T>>,
}

impl<'a, T: Serialize + for<'b> Deserialize<'b> + Default + 'static> Deref for StorageRef<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

pub struct StorageRefMut<'a, T: Serialize + for<'b> Deserialize<'b> + 'static> {
    inner: RefMut<'a, StorageEntry<ClientStorage, T>>,
}

impl<'a, T: Serialize + for<'b> Deserialize<'b> + 'static> Deref for StorageRefMut<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<'a, T: Serialize + for<'b> Deserialize<'b> + 'static> DerefMut for StorageRefMut<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner.data
    }
}

impl<'a, T: Serialize + for<'b> Deserialize<'b> + 'static> Drop for StorageRefMut<'a, T> {
    fn drop(&mut self) {
        self.inner.deref_mut().save();
    }
}

pub struct UsePersistant<T: Serialize + for<'a> Deserialize<'a> + Default + 'static> {
    inner: UseRef<StorageEntry<ClientStorage, T>>,
}

impl<T: Serialize + for<'a> Deserialize<'a> + Default + 'static> UsePersistant<T> {
    pub fn read(&self) -> StorageRef<T> {
        StorageRef {
            inner: self.inner.read(),
        }
    }

    pub fn write(&self) -> StorageRefMut<T> {
        StorageRefMut {
            inner: self.inner.write(),
        }
    }

    pub fn set(&self, value: T) {
        *self.write() = value;
    }

    pub fn modify<F: FnOnce(&mut T)>(&self, f: F) {
        f(&mut self.write());
    }
}

impl<T: Serialize + for<'a> Deserialize<'a> + Default + 'static> Deref for UsePersistant<T> {
    type Target = UseRef<StorageEntry<ClientStorage, T>>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T: Serialize + for<'a> Deserialize<'a> + Default + 'static> DerefMut for UsePersistant<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
