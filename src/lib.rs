//! <div align="center">
//!   <h1>dioxus_storage</h1>
//! </div>
//! <div align="center">
//!   <!-- Crates version -->
//!   <a href="https://crates.io/crates/dioxus_storage">
//!     <img src="https://img.shields.io/crates/v/dioxus_storage.svg?style=flat-square"
//!     alt="Crates.io version" />
//!   </a>
//!   <!-- Downloads -->
//!   <a href="https://crates.io/crates/dioxus_storage">
//!     <img src="https://img.shields.io/crates/d/dioxus_storage.svg?style=flat-square"
//!       alt="Download" />
//!   </a>
//!   <!-- docs -->
//!   <a href="https://docs.rs/dioxus_storage">
//!     <img src="https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square"
//!       alt="docs.rs docs" />
//!   </a>
//! </div>

//! # dioxus-storage

//! A library for handling local storage ergonomically in Dioxus

//! ## Usage

//! ```rust
//! use dioxus_storage::use_storage;
//! use dioxus::prelude::*;

//! fn main() {
//!     dioxus_web::launch(app)
//! }

//! fn app(cx: Scope) -> Element {
//!     let num = use_persistent(cx, "count", || 0);

//!     cx.render(rsx! {
//!         div {
//!             button {
//!                 onclick: move |_| {
//!                     num.modify(|num| *num += 1);
//!                 },
//!                 "Increment"
//!             }
//!             div {
//!                 "{*num.read()}"
//!             }
//!         }
//!     })
//! }
//! ```

mod client_storage;
mod storage;

pub use client_storage::{set_dir_name, set_directory, use_persistent};

pub use once_cell;
pub use postcard;
