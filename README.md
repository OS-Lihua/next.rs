# next.rs

> The AI-native Rust web framework. Pure Rust API. Zero macros. Designed for AI code generation.

[![CI](https://github.com/OS-Lihua/next.rs/actions/workflows/ci.yml/badge.svg)](https://github.com/OS-Lihua/next.rs/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/react-rs-core.svg)](https://crates.io/crates/react-rs-core)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Why next.rs?

next.rs is the first web framework designed for AI to write code in. While other Rust frameworks use macro DSLs that LLMs struggle with, next.rs uses pure method chaining — the pattern AI generates most reliably.

- **Zero macros** — Pure Rust function calls and method chaining. No `view!{}`, no `rsx!{}`, no `html!{}`
- **AI-friendly conventions** — One file, one function, one signature. No ambiguity
- **Full-stack Rust** — SSR on the server, WASM hydration in the browser, one language everywhere
- **Fine-grained reactivity** — Signals, Effects, Memos. Direct DOM updates, no Virtual DOM overhead

```rust
use react_rs_core::create_signal;
use react_rs_elements::html::*;
use react_rs_elements::node::IntoNode;
use react_rs_elements::SignalExt;

pub fn page() -> impl IntoNode {
    let (count, set_count) = create_signal(0);

    div()
        .class("container")
        .child(h1().text("Welcome to next.rs"))
        .child(
            button()
                .text_reactive(count.map(|n| format!("Count: {}", n)))
                .on_click(move |_| { set_count.update(|n| *n += 1); })
        )
}
```

## Quick Start

```bash
next create my-app
cd my-app
next dev
```

`next create` generates a fully working SSR + WASM hydration project. Interactive components (signals, events) work in the browser without configuration.

## Features

### React Core (react-rs)
- Pure Rust API with method chaining (no macros)
- Fine-grained reactivity: Signals, Effects, Memos
- Scope-based effect disposal (no memory leaks)
- Context API for state sharing
- Server-side rendering (SSR)
- WASM runtime with client-side hydration
- Event delegation

### Next.js Features (next-rs)
- File-system based routing (App Router)
- `Link` component and router hooks (`use_router`, `use_pathname`, `use_params`)
- Nested layouts
- Server Actions (register + async execute)
- Tailwind CSS integration
- Static file serving
- WebSocket support
- Dev server with auto browser refresh

### CLI
- `next create` — Project scaffolding with SSR + hydration out of the box
- `next dev` — Dev server with file watching and auto browser refresh
- `next build` — Production build (server binary + WASM)
- `next start` — Production server
- `next add` — Scaffold pages, layouts, components
- `next check --json` — Project validation with machine-readable output

### AI-Native
- `llms.txt` — AI context file so LLMs generate correct next.rs code
- `next check --json` — Structured diagnostics for AI agents
- Zero-macro API — The pattern LLMs generate most reliably
- Deterministic file conventions — No ambiguity for code generation

## Project Structure

```
src/app/
├── layout.rs       # pub fn layout(children: Node) -> impl IntoNode
├── page.rs         # pub fn page() -> impl IntoNode
├── about/
│   └── page.rs     # /about
├── blog/
│   ├── layout.rs   # Blog layout
│   ├── page.rs     # /blog
│   └── [slug]/
│       └── page.rs # /blog/:slug (dynamic route)
└── api/
    └── users/
        └── route.rs # API route /api/users
```

## File Conventions

| File | Export | Signature |
|------|--------|-----------|
| `page.rs` | `pub fn page()` | `-> impl IntoNode` |
| `layout.rs` | `pub fn layout(children: Node)` | `-> impl IntoNode` |
| `route.rs` | HTTP handlers | `ApiRequest -> ApiResponse` |

## Crates

| Crate | Description |
|-------|-------------|
| `react-rs-core` | Signals, Effects, Memos, Context, Scopes |
| `react-rs-elements` | Pure Rust HTML element API |
| `react-rs-dom` | Server-side rendering |
| `react-rs-wasm` | WASM runtime, hydration, event delegation |
| `next-rs-router` | File-system routing, Link component, hooks |
| `next-rs-server` | HTTP server with SSR, WebSocket, static files |
| `next-rs-actions` | Server Actions |
| `next-rs-rsc` | Server component rendering |
| `next-rs-middleware` | Request middleware (path matching) |
| `next-rs-assets` | Image/Font configuration |
| `next-rs-cli` | CLI tool |

## Examples

- `examples/counter` — Reactive counter with signals
- `examples/todo-app` — Todo application with list rendering
- `examples/wasm-demo` — Full SSR + WASM hydration demo

## Roadmap

- [ ] Incremental Static Regeneration (ISR)
- [ ] HTTP Streaming with Suspense boundaries
- [ ] Middleware pipeline integration into server handler
- [ ] Image optimization endpoint (`/_next/image`)
- [ ] AI Context Protocol (`.next-context.json`)
- [ ] Project templates (`next create --template blog`)
- [ ] AI Example Gallery

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

## License

MIT
