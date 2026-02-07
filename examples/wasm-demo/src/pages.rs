use react_rs_elements::html;
use react_rs_elements::node::{IntoNode, Node};

use crate::components::{
    counter_widget, feature_card, greeting_form, navigation, site_footer, todo_list,
};

pub fn render_page(route: &str) -> Node {
    let content = match route {
        "/" => home_page(),
        "/counter" => counter_page(),
        "/about" => about_page(),
        _ => not_found_page(),
    };

    layout(content).into_node()
}

fn layout(content: react_rs_elements::Element) -> react_rs_elements::Element {
    html::div()
        .class("app")
        .child(
            html::header()
                .class("header")
                .child(html::h1().class("logo").text("next.rs"))
                .child(navigation()),
        )
        .child(html::main_el().class("main").child(content))
        .child(site_footer())
}

fn home_page() -> react_rs_elements::Element {
    html::div()
        .class("page home-page")
        .child(
            html::section()
                .class("hero")
                .child(
                    html::h2()
                        .class("hero-title")
                        .text("Next.js, Reimagined in Rust"),
                )
                .child(
                    html::p().class("hero-subtitle").text(
                        "Build blazing-fast web apps with pure Rust. No JavaScript required.",
                    ),
                ),
        )
        .child(
            html::section()
                .class("features")
                .child(html::h2().text("Features"))
                .child(
                    html::div()
                        .class("feature-grid")
                        .child(feature_card(
                            "Pure Rust API",
                            "Build UIs with method chaining. No RSX macros, no JSX.",
                        ))
                        .child(feature_card(
                            "Fine-grained Reactivity",
                            "Signals, Effects, and Memos for optimal performance.",
                        ))
                        .child(feature_card(
                            "SSR + Hydration",
                            "Server-side rendering with seamless client hydration.",
                        ))
                        .child(feature_card(
                            "React Server Components",
                            "Use 'use client' and 'use server' directives.",
                        )),
                ),
        )
        .child(
            html::section()
                .class("demo-section")
                .child(html::h2().text("Live Demos"))
                .child(counter_widget())
                .child(greeting_form()),
        )
}

fn counter_page() -> react_rs_elements::Element {
    html::div()
        .class("page counter-page")
        .child(html::h2().text("Counter Demo"))
        .child(html::p().text("This counter demonstrates reactive state management with Signals."))
        .child(counter_widget())
        .child(
            html::div()
                .class("explanation")
                .child(html::h3().text("How it works"))
                .child(html::p().text(
                    "The counter uses create_signal() to create reactive state. \
                     When buttons are clicked, the signal is updated and the DOM \
                     automatically reflects the new value - no virtual DOM diffing needed!",
                )),
        )
}

fn about_page() -> react_rs_elements::Element {
    html::div()
        .class("page about-page")
        .child(html::h2().text("About next.rs"))
        .child(
            html::div()
                .class("about-content")
                .child(html::p().text(
                    "next.rs is a complete reimplementation of Next.js in Rust, \
                     including a React-like UI framework with pure Rust API.",
                ))
                .child(html::h3().text("Project Goals"))
                .child(
                    html::ul()
                        .child(html::li().text("Pure Rust - no JavaScript runtime needed"))
                        .child(html::li().text("Type-safe UI development"))
                        .child(html::li().text("Excellent performance via WASM"))
                        .child(html::li().text("Full Next.js feature parity")),
                )
                .child(html::h3().text("Current Status"))
                .child(html::p().text("The project includes:"))
                .child(
                    html::ul()
                        .child(html::li().text("react-rs-core: Signal/Effect/Memo/Context"))
                        .child(html::li().text("react-rs-elements: Pure Rust element API"))
                        .child(html::li().text("react-rs-dom: Server-side rendering"))
                        .child(html::li().text("react-rs-wasm: WASM runtime + hydration"))
                        .child(html::li().text("next-rs-router: File-system routing"))
                        .child(html::li().text("next-rs-server: HTTP server"))
                        .child(html::li().text("next-rs-rsc: React Server Components"))
                        .child(html::li().text("next-rs-actions: Server Actions")),
                ),
        )
        .child(todo_list())
}

fn not_found_page() -> react_rs_elements::Element {
    html::div()
        .class("page not-found-page")
        .child(html::h2().text("404 - Page Not Found"))
        .child(html::p().text("The page you're looking for doesn't exist."))
        .child(html::a().href("/").text("Go back home"))
}
