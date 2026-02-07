# next.rs

Next.js reimplemented in Rust, including a React-like UI framework with pure Rust API.

## Features

### React Core (react-rs)
- Pure Rust API with method chaining (no RSX/JSX macros)
- Fine-grained reactivity with Signals, Effects, and Memos
- Context API for state sharing
- Server-side rendering (SSR)

### Next.js Features (next-rs)
- File-system based routing (App Router)
- Nested layouts with loading/error boundaries
- Static Site Generation (SSG)
- Incremental Static Regeneration (ISR)
- HTTP Streaming with Suspense
- React Server Components
- Server Actions
- Middleware
- Image and Font optimization

## Quick Start

```bash
# Create a new project
next-rs create my-app
cd my-app

# Start development server
next-rs dev

# Build for production
next-rs build

# Start production server
next-rs start
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
app/
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

## Crates

| Crate | Description |
|-------|-------------|
| `react-rs-core` | Signal/Effect/Memo/Context reactivity system |
| `react-rs-elements` | Pure Rust element API (div, span, etc.) |
| `react-rs-dom` | Server-side rendering |
| `next-rs-router` | File-system routing and layouts |
| `next-rs-server` | HTTP server with SSR/SSG/ISR |
| `next-rs-rsc` | React Server Components |
| `next-rs-actions` | Server Actions |
| `next-rs-middleware` | Request middleware |
| `next-rs-assets` | Image/Font optimization |
| `next-rs-cli` | CLI tool |

## Examples

- `examples/hello-world` - Basic hello world
- `examples/counter` - Reactive counter with signals
- `examples/todo-app` - Todo application
- `examples/blog` - Blog with App Router

## License

MIT
