use std::{
    any::TypeId,
    sync::{Arc, RwLock},
};

use dioxus::prelude::*;
use once_cell::sync::OnceCell;
use postcard::to_allocvec;
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};

static STORAGE: OnceCell<PersistantStorageContext> = OnceCell::new();

pub fn use_init_storage(cx: Scope) {
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

        let storage = PersistantStorageContext { storage };
        STORAGE.set(storage.clone()).unwrap();
        storage
    });
}

#[allow(clippy::needless_return)]
pub fn get_data() -> String {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let serialized = serde_to_string(&*STORAGE.get().unwrap().storage.read().unwrap());
        return format!(r#"<div id="dioxus-storage" data-serialized="{serialized}"></div>"#);
    }
    #[cfg(target_arch = "wasm32")]
    return "".to_string();
}

#[cfg(not(target_arch = "wasm32"))]
fn serde_to_string<T: Serialize>(value: &T) -> String {
    let serialized = to_allocvec(value).unwrap();
    println!("serialized: {:?}", serialized.len());
    let compressed = yazi::compress(
        &serialized,
        yazi::Format::Zlib,
        yazi::CompressionLevel::BestSize,
    )
    .unwrap();
    println!("compressed: {:?}", compressed.len());
    let as_str: String = compressed
        .iter()
        .flat_map(|u| {
            [
                char::from_digit(((*u & 0xF0) >> 4).into(), 16).unwrap(),
                char::from_digit((*u & 0x0F).into(), 16).unwrap(),
            ]
            .into_iter()
        })
        .collect();
    as_str
}

#[cfg(target_arch = "wasm32")]
fn serde_from_string<T: for<'a> Deserialize<'a>>(value: &str) -> T {
    let mut bytes: Vec<u8> = Vec::new();
    let mut chars = value.chars();
    while let Some(c) = chars.next() {
        let n1 = c.to_digit(16).unwrap();
        let c2 = chars.next().unwrap();
        let n2 = c2.to_digit(16).unwrap();
        bytes.push((n1 * 16 + n2) as u8);
    }
    let (decompressed, _) = yazi::decompress(&bytes, yazi::Format::Zlib).unwrap();
    postcard::from_bytes(&decompressed).unwrap()
}

pub fn server_state<T: 'static + Serialize + for<'a> Deserialize<'a>>(
    cx: Scope,
    name: &'static str,
    init: impl FnOnce() -> T,
) -> T {
    let context: PersistantStorageContext = cx
        .consume_context()
        .expect("use_server_state must be called inside a context that contains InitStorage");
    #[cfg(target_arch = "wasm32")]
    return deserialize_server_state(context, name);
    #[cfg(not(target_arch = "wasm32"))]
    return serialize_server_state(context, name, init);
}

#[cfg(not(target_arch = "wasm32"))]
fn serialize_server_state<T: 'static + Serialize + for<'a> Deserialize<'a>>(
    context: PersistantStorageContext,
    name: &'static str,
    init: impl FnOnce() -> T,
) -> T {
    let value = init();
    context.set(name, &value);
    value
}

#[cfg(target_arch = "wasm32")]
fn deserialize_server_state<T: 'static + Serialize + for<'a> Deserialize<'a>>(
    context: PersistantStorageContext,
    name: &'static str,
) -> T {
    context.get(name).expect("state not found")
}

#[derive(Clone, Debug)]
struct PersistantStorageContext {
    storage: Arc<RwLock<PersistantStorage>>,
}

impl PersistantStorageContext {
    #[cfg(target_arch = "wasm32")]
    fn get<T: 'static + for<'a> Deserialize<'a>>(&self, name: &str) -> Option<T> {
        let storage = self.storage.read().ok()?;
        let data = storage.data.get(name)?;
        let data = postcard::from_bytes(data).unwrap();
        Some(data)
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn set<T: 'static + Serialize>(&self, name: &str, value: &T) {
        let data = to_allocvec(&value).unwrap();
        let mut storage = self.storage.write().unwrap();
        storage.data.insert(name.to_string(), data);
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
struct PersistantStorage {
    data: FxHashMap<String, Vec<u8>>,
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq, Hash)]
struct SerializeTypeId {
    data: u64,
}

impl From<TypeId> for SerializeTypeId {
    fn from(id: TypeId) -> Self {
        Self {
            data: unsafe { std::mem::transmute(id) },
        }
    }
}

impl From<SerializeTypeId> for TypeId {
    fn from(id: SerializeTypeId) -> Self {
        unsafe { std::mem::transmute(id.data) }
    }
}
