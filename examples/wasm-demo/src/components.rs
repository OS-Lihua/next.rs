use react_rs_core::{create_resource, create_signal, provide_context, Resource};
use react_rs_elements::html;
use react_rs_elements::node::{each, IntoNode, Node};
use react_rs_elements::reactive::SignalExt;
use react_rs_elements::{error_boundary, suspense, Element};

pub fn navigation() -> Element {
    html::nav().class("nav").child(
        html::ul()
            .class("nav-list")
            .child(html::li().child(html::a().href("/").text("Home")))
            .child(html::li().child(html::a().href("/counter").text("Counter")))
            .child(html::li().child(html::a().href("/about").text("About"))),
    )
}

pub fn counter_widget() -> Node {
    let (count, set_count) = create_signal(0);

    let inc = set_count.clone();
    let dec = set_count.clone();
    let reset = set_count.clone();
    let high_count = count.map(|n| *n > 5);

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
        .child(
            html::p()
                .class("high-count-msg")
                .text("High count!")
                .show_when(high_count),
        )
        .into_node()
}

pub fn todo_list() -> Node {
    let (todos, set_todos) = create_signal(vec![
        "Learn Rust".to_string(),
        "Build with next.rs".to_string(),
        "Deploy to production".to_string(),
    ]);
    let (input_val, set_input) = create_signal(String::new());

    let add_todos = set_todos.clone();
    let add_input = input_val.clone();
    let clear_input = set_input.clone();

    let count_signal = todos.clone();

    html::div()
        .class("todo-widget")
        .child(html::h3().text("Dynamic Todo List"))
        .child(
            html::div()
                .class("todo-input-row")
                .child(
                    html::input()
                        .type_("text")
                        .placeholder("Add a todo...")
                        .class("form-input todo-input")
                        .bind_value(input_val, set_input),
                )
                .child(
                    html::button()
                        .class("btn btn-add")
                        .text("Add")
                        .on_click(move |_| {
                            let value = add_input.get_untracked();
                            if !value.is_empty() {
                                add_todos.update(|t| t.push(value));
                                clear_input.set(String::new());
                            }
                        }),
                ),
        )
        .child(each(todos, |item, _i| {
            html::li()
                .class("todo-item")
                .text(item.as_str())
                .into_node()
        }))
        .child(
            html::p()
                .class("todo-count")
                .text("Total: ")
                .child(html::span().text_reactive(count_signal.map(|t| t.len().to_string())))
                .child(html::span().text(" items")),
        )
        .into_node()
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

pub fn data_loading_demo() -> Node {
    let resource: Resource<Vec<String>> = create_resource();
    let (users, set_users) = create_signal(Vec::<String>::new());

    let load_res = resource.clone();
    let load_users = set_users.clone();
    let error_res = resource.clone();

    let load = move |_: react_rs_elements::events::Event| {
        load_res.set_ready(vec![
            "Alice".to_string(),
            "Bob".to_string(),
            "Charlie".to_string(),
        ]);
        load_users.set(vec![
            "Alice".to_string(),
            "Bob".to_string(),
            "Charlie".to_string(),
        ]);
    };

    let trigger_error = move |_: react_rs_elements::events::Event| {
        error_res.set_error("Simulated network timeout");
    };

    html::div()
        .class("data-demo")
        .child(html::h3().text("Resource + Suspense + ErrorBoundary"))
        .child(
            html::div()
                .class("counter-buttons")
                .child(
                    html::button()
                        .class("btn btn-increment")
                        .text("Load Data")
                        .on_click(load),
                )
                .child(
                    html::button()
                        .class("btn btn-decrement")
                        .text("Trigger Error")
                        .on_click(trigger_error),
                ),
        )
        .child(error_boundary(
            &resource,
            |err| {
                html::p()
                    .class("error-msg")
                    .text(format!("Error: {}", err))
                    .into_node()
            },
            suspense(
                &resource,
                html::p().class("loading-msg").text("Loading data..."),
                each(users, |item, _| {
                    html::li()
                        .class("todo-item")
                        .text(item.as_str())
                        .into_node()
                }),
            ),
        ))
        .into_node()
}

#[derive(Clone, Debug)]
pub struct AppTheme {
    pub name: String,
}

pub fn context_demo() -> Node {
    provide_context(AppTheme {
        name: "Default".to_string(),
    });

    let (theme_name, set_theme) = create_signal("Default".to_string());

    let set_dark = set_theme.clone();
    let set_light = set_theme.clone();

    html::div()
        .class("context-demo")
        .child(html::h3().text("Context API"))
        .child(
            html::p().text("Current theme: ").child(
                html::span()
                    .class("greeting-name")
                    .text_reactive(theme_name.map(|n| n.clone())),
            ),
        )
        .child(
            html::div()
                .class("counter-buttons")
                .child(
                    html::button()
                        .class("btn btn-increment")
                        .text("Dark Theme")
                        .on_click(move |_| {
                            provide_context(AppTheme {
                                name: "Dark".to_string(),
                            });
                            set_dark.set("Dark".to_string());
                        }),
                )
                .child(
                    html::button()
                        .class("btn btn-reset")
                        .text("Light Theme")
                        .on_click(move |_| {
                            provide_context(AppTheme {
                                name: "Light".to_string(),
                            });
                            set_light.set("Light".to_string());
                        }),
                ),
        )
        .into_node()
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
