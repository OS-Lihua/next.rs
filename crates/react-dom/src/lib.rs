//! # react-rs-dom
//!
//! Server-side rendering for react.rs components.
//!
//! This crate provides [`render_to_string()`] which converts a component tree
//! into HTML string output for server-side rendering (SSR).
//!
//! For client-side rendering and hydration, use `react-rs-wasm` instead.

mod render;

pub use render::{render_to_string, RenderOutput};
