use react_rs_core::{create_resource, ResourceState};
use react_rs_elements::html::*;
use react_rs_elements::node::IntoNode;

pub fn user_profile() -> impl IntoNode {
    let user = create_resource::<String>();

    user.set_ready("Alice".to_string());

    match user.read() {
        ResourceState::Loading => div().text("Loading user...").into_node(),
        ResourceState::Ready(name) => div()
            .child(h2().text(format!("Hello, {}!", name)))
            .child(p().text("Your profile is loaded."))
            .into_node(),
        ResourceState::Error(e) => div()
            .class("text-red-500")
            .text(format!("Error: {}", e))
            .into_node(),
    }
}
