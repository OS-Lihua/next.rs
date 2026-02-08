use react_rs_elements::html::*;
use react_rs_elements::node::{IntoNode, Node};

pub fn card(title: &str, children: Node) -> impl IntoNode {
    div()
        .class("border rounded-lg shadow p-4")
        .child(h3().class("text-lg font-bold mb-2").text(title))
        .child(children)
}

pub fn badge(text: &str, _color: &str) -> impl IntoNode {
    span().class("px-2 py-1 rounded text-sm").text(text)
}

pub fn page() -> impl IntoNode {
    div()
        .class("space-y-4 p-8")
        .child(card(
            "User Info",
            div()
                .child(p().text("John Doe"))
                .child(badge("Admin", "blue"))
                .into_node(),
        ))
        .child(card(
            "Statistics",
            div()
                .child(p().text("Total orders: 42"))
                .child(badge("Active", "green"))
                .into_node(),
        ))
}
