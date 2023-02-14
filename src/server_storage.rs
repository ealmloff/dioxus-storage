use crate::storage::*;
use dioxus::prelude::*;
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};

#[derive(Clone, Default, Debug)]
struct ServerStorage;

static STORAGE: OnceCell<PersistantStorageContext<ServerStorage>> = OnceCell::new();

pub fn use_init_storage(cx: &ScopeState) {
    use_context_provider(cx, || {
        #[cfg(target_arch = "wasm32")]
        let storage: Arc<RwLock<PersistantStorage>> = Arc::new(RwLock::new(serde_from_string(
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
        let storage = Arc::new(RwLock::new(PersistantStorage::default()));

        let storage = PersistantStorageContext {
            storage,
            ..Default::default()
        };
        STORAGE.set(storage.clone()).unwrap();
        storage
    });
}

#[allow(clippy::needless_return)]
pub fn get_data() -> String {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let serialized = serde_to_string(&*STORAGE.get().unwrap().storage.read().unwrap());
        return format!(
            r#"<meta id="dioxus-storage" data-serialized="{serialized}" hidden="true"/>"#
        );
    }
    #[cfg(target_arch = "wasm32")]
    return "".to_string();
}

pub fn server_state<T: 'static + Serialize + for<'a> Deserialize<'a>>(
    cx: &ScopeState,
    init: impl FnOnce() -> T,
) -> T {
    let context: PersistantStorageContext<ServerStorage> = cx
        .consume_context()
        .expect("use_server_state must be called inside a context that contains InitStorage");
    #[cfg(target_arch = "wasm32")]
    return deserialize_server_state(context);
    #[cfg(not(target_arch = "wasm32"))]
    return serialize_server_state(context, init);
}

#[cfg(not(target_arch = "wasm32"))]
fn serialize_server_state<T: 'static + Serialize + for<'a> Deserialize<'a>>(
    context: PersistantStorageContext<ServerStorage>,
    init: impl FnOnce() -> T,
) -> T {
    let value = init();
    context.set(&value);
    value
}

#[cfg(target_arch = "wasm32")]
fn deserialize_server_state<T: 'static + Serialize + for<'a> Deserialize<'a>>(
    context: PersistantStorageContext<ServerStorage>,
) -> T {
    context.get().expect("state not found")
}
