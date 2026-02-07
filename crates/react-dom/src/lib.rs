mod render;

#[cfg(target_arch = "wasm32")]
mod dom;

#[cfg(target_arch = "wasm32")]
mod events;

#[cfg(target_arch = "wasm32")]
mod hydrate;

pub use render::{render_to_string, RenderOutput};

#[cfg(target_arch = "wasm32")]
pub use dom::{mount, mount_to_body};

#[cfg(target_arch = "wasm32")]
pub use hydrate::hydrate;
