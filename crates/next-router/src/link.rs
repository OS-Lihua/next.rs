use react_rs_elements::html;
use react_rs_elements::Element;

pub struct Link {
    href: String,
    children_text: Option<String>,
    class: Option<String>,
    prefetch: bool,
}

impl Link {
    pub fn new(href: impl Into<String>) -> Self {
        Self {
            href: href.into(),
            children_text: None,
            class: None,
            prefetch: true,
        }
    }

    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.children_text = Some(text.into());
        self
    }

    pub fn class(mut self, class: impl Into<String>) -> Self {
        self.class = Some(class.into());
        self
    }

    pub fn prefetch(mut self, prefetch: bool) -> Self {
        self.prefetch = prefetch;
        self
    }

    pub fn href(&self) -> &str {
        &self.href
    }

    pub fn build(self) -> Element {
        let mut el = html::a().href(&self.href).attr("data-link", "true");

        if self.prefetch {
            el = el.attr("data-prefetch", "true");
        }

        if let Some(class) = self.class {
            el = el.class(&class);
        }

        if let Some(text) = self.children_text {
            el = el.text(&text);
        }

        el
    }
}

pub fn link(href: impl Into<String>) -> Link {
    Link::new(href)
}

#[cfg(test)]
mod tests {
    use super::*;
    use react_rs_elements::node::IntoNode;

    #[test]
    fn test_link_basic() {
        let l = link("/about").text("About").build();
        assert_eq!(l.tag(), "a");
        let node = l.into_node();
        let html = react_rs_dom::render_to_string(&node);
        assert!(html.html.contains("href=\"/about\""));
        assert!(html.html.contains("data-link=\"true\""));
        assert!(html.html.contains("About"));
    }

    #[test]
    fn test_link_with_class() {
        let l = link("/home").text("Home").class("nav-link").build();
        let node = l.into_node();
        let html = react_rs_dom::render_to_string(&node);
        assert!(html.html.contains("class=\"nav-link\""));
    }

    #[test]
    fn test_link_no_prefetch() {
        let l = link("/heavy").text("Heavy Page").prefetch(false).build();
        let node = l.into_node();
        let html = react_rs_dom::render_to_string(&node);
        assert!(!html.html.contains("data-prefetch"));
    }

    #[test]
    fn test_link_prefetch_default() {
        let l = link("/fast").text("Fast").build();
        let node = l.into_node();
        let html = react_rs_dom::render_to_string(&node);
        assert!(html.html.contains("data-prefetch=\"true\""));
    }
}
