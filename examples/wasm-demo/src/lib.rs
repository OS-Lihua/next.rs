pub mod components;
pub mod pages;

use react_rs_elements::node::Node;

pub fn render_app(route: &str) -> Node {
    pages::render_page(route)
}

#[cfg(target_arch = "wasm32")]
mod wasm {
    use super::*;
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen(start)]
    pub fn hydrate() {
        let window = web_sys::window().expect("no window");
        let location = window.location();
        let pathname = location.pathname().unwrap_or_else(|_| "/".to_string());

        web_sys::console::log_1(&format!("Hydrating route: {}", pathname).into());

        react_rs_wasm::setup_link_interception(Box::new(|path| {
            web_sys::console::log_1(&format!("Navigating to: {}", path).into());
            let node = render_app(&path);
            let _ = react_rs_wasm::mount(&node, "app");
        }));

        let node = render_app(&pathname);
        match react_rs_wasm::hydrate(&node, "app") {
            Ok(_) => web_sys::console::log_1(&"Hydration successful!".into()),
            Err(e) => {
                web_sys::console::error_1(&format!("Hydration failed: {:?}, mounting fresh", e).into());
                let _ = react_rs_wasm::mount(&node, "app");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use react_rs_dom::render_to_string;

    #[test]
    fn test_render_home() {
        let node = render_app("/");
        let output = render_to_string(&node);
        assert!(output.html.contains("next.rs"));
    }

    #[test]
    fn test_render_counter() {
        let node = render_app("/counter");
        let output = render_to_string(&node);
        assert!(output.html.contains("Counter"));
    }

    #[test]
    fn test_render_about() {
        let node = render_app("/about");
        let output = render_to_string(&node);
        assert!(output.html.contains("About"));
    }
}
