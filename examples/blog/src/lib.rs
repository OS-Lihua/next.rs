pub mod app;
pub mod components;
pub mod data;
pub mod rsc;

pub use app::layout::root_layout;
pub use app::page::home_page;
pub use app::posts::posts_page;
pub use rsc::{render_async_home_page_rsc, render_home_page_rsc, render_post_page_rsc};

#[cfg(test)]
mod tests {
    use super::*;
    use react_rs_dom::render_to_string;
    use react_rs_elements::node::IntoNode;

    #[test]
    fn test_home_page_renders() {
        let output = render_to_string(&home_page().into_node());
        assert!(output.html.contains("Welcome to the Blog"));
    }

    #[test]
    fn test_posts_page_renders() {
        let output = render_to_string(&posts_page().into_node());
        assert!(output.html.contains("Blog Posts"));
    }

    #[test]
    fn test_layout_renders() {
        use react_rs_elements::html::*;
        let content = div().text("Test content");
        let output = render_to_string(&root_layout(content).into_node());
        assert!(output.html.contains("next.rs"));
        assert!(output.html.contains("Test content"));
    }
}
