use react_rs_core::create_signal;
use react_rs_elements::html;
use react_rs_elements::reactive::SignalExt;
use react_rs_elements::Element;

pub fn navigation() -> Element {
    html::nav().class("nav").child(
        html::ul()
            .class("nav-list")
            .child(html::li().child(html::a().href("/").text("Home")))
            .child(html::li().child(html::a().href("/counter").text("Counter")))
            .child(html::li().child(html::a().href("/about").text("About"))),
    )
}

pub fn counter_widget() -> Element {
    let (count, set_count) = create_signal(0);

    let inc = set_count.clone();
    let dec = set_count.clone();
    let reset = set_count.clone();

    html::div()
        .class("counter-widget")
        .child(
            html::div()
                .class("counter-display")
                .child(html::span().class("count-label").text("Count: "))
                .child(
                    html::span()
                        .class("count-value")
                        .text_reactive(count.map(|n| n.to_string())),
                ),
        )
        .child(
            html::div()
                .class("counter-buttons")
                .child(
                    html::button()
                        .class("btn btn-decrement")
                        .text("-")
                        .on_click(move |_| dec.update(|n| *n -= 1)),
                )
                .child(
                    html::button()
                        .class("btn btn-reset")
                        .text("Reset")
                        .on_click(move |_| reset.set(0)),
                )
                .child(
                    html::button()
                        .class("btn btn-increment")
                        .text("+")
                        .on_click(move |_| inc.update(|n| *n += 1)),
                ),
        )
}

pub fn todo_list() -> Element {
    let (count, set_count) = create_signal(3);

    let add_todo = set_count.clone();

    html::div()
        .class("todo-widget")
        .child(html::h3().text("Quick Todo"))
        .child(
            html::ul()
                .class("todo-list")
                .child(html::li().class("todo-item").text("Learn Rust"))
                .child(html::li().class("todo-item").text("Build with next.rs"))
                .child(html::li().class("todo-item").text("Deploy to production")),
        )
        .child(
            html::p()
                .class("todo-count")
                .text("Total: ")
                .child(html::span().text_reactive(count.map(|n| n.to_string())))
                .child(html::span().text(" items")),
        )
        .child(
            html::button()
                .class("btn btn-add")
                .text("Add Todo")
                .on_click(move |_| {
                    add_todo.update(|n| *n += 1);
                }),
        )
}

pub fn greeting_form() -> Element {
    let (name, set_name) = create_signal(String::new());

    let display_name = name.clone();

    html::div()
        .class("greeting-form")
        .child(html::h3().text("Greeting Form"))
        .child(
            html::div()
                .class("form-group")
                .child(html::label().text("Your name:"))
                .child(
                    html::input()
                        .type_("text")
                        .placeholder("Type your name...")
                        .class("form-input")
                        .bind_value(name, set_name),
                ),
        )
        .child(
            html::p()
                .class("greeting-output")
                .text("Hello, ")
                .child(
                    html::span()
                        .class("greeting-name")
                        .text_reactive(display_name.map(|n| {
                            if n.is_empty() {
                                "World".to_string()
                            } else {
                                n.clone()
                            }
                        })),
                )
                .child(html::span().text("!")),
        )
}

pub fn feature_card(title: &str, description: &str) -> Element {
    html::div()
        .class("feature-card")
        .child(html::h3().class("feature-title").text(title))
        .child(html::p().class("feature-description").text(description))
}

pub fn site_footer() -> Element {
    html::footer().class("footer").child(
        html::p()
            .text("Built with next.rs - Next.js reimplemented in Rust")
            .child(html::span().text(" | "))
            .child(
                html::a()
                    .href("https://github.com/OS-Lihua/next.rs")
                    .text("GitHub"),
            ),
    )
}
