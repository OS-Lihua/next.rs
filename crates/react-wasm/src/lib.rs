//! # react-rs-wasm
//!
//! Client-side WASM runtime for react.rs applications.
//!
//! Provides DOM mounting, hydration, event delegation, client-side routing,
//! and React Server Components runtime.
//!
//! For server-side rendering, use `react-rs-dom` instead.

mod dom;
pub mod fetch;
mod hydration;
mod router;
mod runtime;
pub mod websocket;

pub use dom::{
    mount, register_event_handler, render_to_dom, unregister_event_handler, DomNode, WasmEvent,
};
pub use hydration::{hydrate, hydrate_client_components, HydrationError, HydrationResult};
pub use router::{back, forward, navigate, replace, setup_link_interception, use_location, Router};
pub use runtime::{ClientComponentRegistry, RscRuntime};
pub use websocket::{use_websocket, use_websocket_simple, WsHandle};

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! From react.rs WASM runtime.", name)
}
