use react_rs_core::create_signal;
use react_rs_elements::html::*;
use react_rs_elements::node::IntoNode;
use react_rs_elements::reactive::SignalExt;

pub fn counter() -> react_rs_elements::Element {
    let (count, set_count) = create_signal(0);

    let inc = set_count.clone();
    let dec = set_count.clone();

    div()
        .class("counter")
        .child(h1().text("Counter Example"))
        .child(p().text_reactive(count.map(|n| format!("Count: {}", n))))
        .child(
            div()
                .class("buttons")
                .child(
                    button()
                        .text("-")
                        .on_click(move |_| dec.update(|n| *n -= 1)),
                )
                .child(
                    button()
                        .text("+")
                        .on_click(move |_| inc.update(|n| *n += 1)),
                ),
        )
}

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn main() {
    react_rs_wasm::mount(&counter().into_node(), "root").expect("Failed to mount counter");
}

#[cfg(not(target_arch = "wasm32"))]
pub fn main() {
    let output = react_rs_dom::render_to_string(&counter().into_node());
    println!("{}", output.html);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_counter_renders() {
        let output = react_rs_dom::render_to_string(&counter().into_node());
        assert!(output.html.contains("Counter Example"));
        assert!(output.html.contains("Count: 0"));
    }
}
