use crate::storage::*;
use once_cell::sync::Lazy;
use postcard::to_allocvec;
use serde::{Deserialize, Serialize};
use std::{
    marker::PhantomData,
    sync::{Arc, RwLock},
};

static STORAGE: Lazy<PersistentStorageContext<ServerStorage>> = Lazy::new(|| {
    #[cfg(target_arch = "wasm32")]
    let storage: Arc<RwLock<PersistentStorage>> = Arc::new(RwLock::new(serde_from_string(
        &web_sys::window()
            .expect("should have a window")
            .document()
            .expect("should have a document")
            .get_element_by_id("dioxus-storage")
            .expect("should have a dioxus-storage element")
            .get_attribute("data-serialized")
            .expect("should have a dioxus-storage element with data-serialized attribute"),
    )));
    #[cfg(not(target_arch = "wasm32"))]
    let storage = Arc::new(RwLock::new(PersistentStorage::default()));

    PersistentStorageContext {
        storage,
        ..Default::default()
    }
});

#[derive(Clone, Debug, Default)]
pub struct PersistentStorageContext<T> {
    pub storage: Arc<RwLock<PersistentStorage>>,
    pub phantom: PhantomData<T>,
}

impl<C> PersistentStorageContext<C> {
    pub fn get<T: for<'a> Deserialize<'a>>(&self) -> Option<T> {
        let mut storage = self.storage.write().ok()?;
        let idx = storage.idx;
        storage.idx += 1;
        let data = storage.data.get(idx)?;
        let data = postcard::from_bytes(data).unwrap();
        Some(data)
    }

    pub fn set<T: Serialize>(&self, value: &T) {
        let data = to_allocvec(&value).unwrap();
        let mut storage = self.storage.write().unwrap();
        storage.data.push(data);
    }
}

#[derive(Clone, Debug, Default)]
struct ServerStorage;

impl StorageBacking for ServerStorage {
    type Key = ();

    fn set<T: Serialize>(_: Self::Key, value: &T) {
        STORAGE.set(value);
    }

    #[allow(clippy::needless_return)]
    fn get<T: for<'a> Deserialize<'a>>(_: &Self::Key) -> Option<T> {
        #[cfg(target_arch = "wasm32")]
        return STORAGE.get();
        #[cfg(not(target_arch = "wasm32"))]
        return None;
    }
}

#[allow(clippy::needless_return)]
pub fn get_data() -> String {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let serialized = serde_to_string(&*STORAGE.storage.read().unwrap());
        return format!(
            r#"<meta id="dioxus-storage" data-serialized="{serialized}" hidden="true"/>"#
        );
    }
    #[cfg(target_arch = "wasm32")]
    return "".to_string();
}

pub fn server_state<T: 'static + Serialize + for<'a> Deserialize<'a>>(
    init: impl FnOnce() -> T,
) -> T {
    storage_entry::<ServerStorage, T>((), init)
}

#[macro_export]
macro_rules! server_state {
    ($f: expr) => {{
        let r;
        #[cfg(not(target_arch = "wasm32"))]
        {
            r = server_state($f);
        }
        #[cfg(target_arch = "wasm32")]
        {
            r = server_state(|| panic!("server state not found"));
        }
        r
    }};
}
