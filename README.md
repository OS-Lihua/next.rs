# next.rs

[![CI](https://github.com/OS-Lihua/next.rs/actions/workflows/ci.yml/badge.svg)](https://github.com/OS-Lihua/next.rs/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/OS-Lihua/next.rs/graph/badge.svg)](https://codecov.io/gh/OS-Lihua/next.rs)
[![Crates.io](https://img.shields.io/crates/v/react-rs-core.svg)](https://crates.io/crates/react-rs-core)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Next.js reimplemented in Rust, including a React-like UI framework with pure Rust API. Zero unsafe code. All 11 crates published on [crates.io](https://crates.io/search?q=react-rs).

## Features

### React Core (react-rs)
- Pure Rust API with method chaining (no RSX/JSX macros)
- Fine-grained reactivity with Signals, Effects, and Memos
- Context API for state sharing
- Server-side rendering (SSR)
- WASM runtime with client-side hydration
- Event delegation (zero memory leaks)

### Next.js Features (next-rs)
- File-system based routing (App Router)
- `Link` component and `use_router` / `use_pathname` / `use_params` hooks
- Nested layouts with loading/error boundaries
- Static Site Generation (SSG)
- Incremental Static Regeneration (ISR)
- HTTP Streaming with Suspense
- React Server Components with `use_client!` / `use_server!` directives
- Server Actions
- Middleware
- Image and Font optimization

## Quick Start

```bash
# Create a new project
next create my-app
cd my-app

# Start development server
next dev

# Build for production
next build

# Start production server
next start
```

## React Example

```rust
use react_rs_core::create_signal;
use react_rs_elements::html::*;

fn counter() -> Element {
    let (count, set_count) = create_signal(0);
    
    div()
        .class("counter")
        .child(h1().text_reactive(count.map(|n| format!("Count: {}", n))))
        .child(
            button()
                .text("Increment")
                .on_click(move |_| set_count.update(|n| *n + 1))
        )
}
```

## Project Structure

```
src/app/
├── layout.rs           # Root layout
├── page.rs             # Home page (/)
├── loading.rs          # Loading state
├── error.rs            # Error boundary
├── not-found.rs        # 404 page
│
├── about/
│   └── page.rs         # /about
│
├── blog/
│   ├── layout.rs       # Blog layout
│   ├── page.rs         # /blog
│   └── [slug]/
│       └── page.rs     # /blog/:slug (dynamic route)
│
└── api/
    └── users/
        └── route.rs    # API route /api/users
```

## File Conventions

Every file follows a strict convention — one way to do things:

| File | Export | Signature |
|------|--------|-----------|
| `page.rs` | `pub fn page()` | `-> impl IntoNode` |
| `layout.rs` | `pub fn layout(children: Node)` | `-> impl IntoNode` |
| `loading.rs` | `pub fn loading()` | `-> impl IntoNode` |
| `error.rs` | `pub fn error(msg: String)` | `-> impl IntoNode` |
| `route.rs` | HTTP handlers | `ApiRequest -> ApiResponse` |

## Crates

| Crate | Description |
|-------|-------------|
| `react-rs-core` | Signal/Effect/Memo/Context reactivity system |
| `react-rs-elements` | Pure Rust element API (div, span, etc.) |
| `react-rs-dom` | Server-side rendering |
| `react-rs-wasm` | WASM runtime, hydration, event delegation |
| `next-rs-router` | File-system routing, Link component, hooks |
| `next-rs-server` | HTTP server with SSR/SSG/ISR |
| `next-rs-rsc` | React Server Components, use_client!/use_server! |
| `next-rs-actions` | Server Actions |
| `next-rs-middleware` | Request middleware |
| `next-rs-assets` | Image/Font optimization |
| `next-rs-cli` | CLI tool |

## Examples

- `examples/hello-world` - Basic hello world
- `examples/counter` - Reactive counter with signals
- `examples/todo-app` - Todo application
- `examples/blog` - Blog with App Router
- `examples/wasm-demo` - Full SSR + WASM hydration demo with routing

## Benchmarks

```
signal_create             17.3 ns
signal_get                0.27 ns
signal_set                2.8 ns
signal_set + 1 effect     9.8 ns
signal_set + 10 effects   108 ns
effect_create             16.8 ns
memo_get                  2.1 ns
deep_chain_3_memos        32 ns
```

Run with: `cargo bench -p react-rs-core`

## License

MIT
