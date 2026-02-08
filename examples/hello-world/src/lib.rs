use react_rs_elements::html::*;
use react_rs_elements::node::IntoNode;

pub fn app() -> react_rs_elements::Element {
    div()
        .class("container")
        .child(h1().text("Hello, react.rs!"))
        .child(p().text("A React-like framework for Rust"))
        .child(
            ul().child(li().text("Pure Rust API"))
                .child(li().text("No macros needed"))
                .child(li().text("Full reactivity")),
        )
}

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn main() {
    let node = app().into_node();
    let _ = react_rs_wasm::mount(&node, "app");
}

#[cfg(not(target_arch = "wasm32"))]
pub fn main() {
    let output = react_rs_dom::render_to_string(&app().into_node());
    println!("{}", output.html);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_renders() {
        let output = react_rs_dom::render_to_string(&app().into_node());
        assert!(output.html.contains("Hello, react.rs!"));
        assert!(output.html.contains("Pure Rust API"));
    }
}
