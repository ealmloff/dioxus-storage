use postcard::to_allocvec;
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;
use std::sync::{Arc, RwLock};

#[cfg(not(target_arch = "wasm32"))]
pub fn serde_to_string<T: Serialize>(value: &T) -> String {
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
pub fn serde_from_string<T: for<'a> Deserialize<'a>>(value: &str) -> T {
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

#[derive(Clone, Debug, Default)]
pub struct PersistantStorageContext<T> {
    pub storage: Arc<RwLock<PersistantStorage>>,
    pub phantom: PhantomData<T>,
}

impl<C> PersistantStorageContext<C> {
    #[cfg(target_arch = "wasm32")]
    pub fn get<T: 'static + for<'a> Deserialize<'a>>(&self) -> Option<T> {
        let mut storage = self.storage.write().ok()?;
        let idx = storage.idx;
        storage.idx += 1;
        let data = storage.data.get(idx)?;
        let data = postcard::from_bytes(data).unwrap();
        Some(data)
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn set<T: 'static + Serialize>(&self, value: &T) {
        let data = to_allocvec(&value).unwrap();
        let mut storage = self.storage.write().unwrap();
        storage.data.push(data);
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PersistantStorage {
    pub data: Vec<Vec<u8>>,
    pub idx: usize,
}
