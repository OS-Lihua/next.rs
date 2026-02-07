mod dom;
mod hydration;
mod runtime;

pub use dom::{mount, register_event_handler, render_to_dom, unregister_event_handler, DomNode};
pub use hydration::{hydrate, hydrate_client_components, HydrationError, HydrationResult};
pub use runtime::{ClientComponentRegistry, RscRuntime};

use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn init() {}

#[wasm_bindgen]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! From react.rs WASM runtime.", name)
}
