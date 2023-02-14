use crate::storage::*;
use dioxus::prelude::*;
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    hash::Hash,
    sync::{Arc, RwLock},
};
use wasm_bindgen::JsCast;
use web_sys::{window, Document, HtmlDocument};

#[derive(Clone, Default, Debug)]
struct ServerStorage;

fn html_document() -> HtmlDocument {
    window().unwrap().document().unwrap().unchecked_into()
}

fn cookies() -> HashMap<String, String> {
    html_document()
        .cookie()
        .ok()
        .map(|c| {
            c.split(";")
                .filter_map(|seg| seg.trim().split_once("="))
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect()
        })
        .unwrap_or_default()
}

fn set_cookies(cookies: HashMap<String, String>) {
    let string: String = cookies.iter().map(|(k, v)| format!("{k}={v}; ")).collect();
    let _ = html_document().set_cookie(&string);
}

fn set_cookie(key: String, value: String) {
    let mut old = cookies();
    old.insert(key, value);
    set_cookies(old);
}
