#[derive(Debug, Clone, Default)]
pub struct Head {
    pub title: Option<String>,
    pub meta_tags: Vec<MetaTag>,
    pub links: Vec<LinkTag>,
}

#[derive(Debug, Clone)]
pub struct MetaTag {
    pub name: String,
    pub content: String,
}

#[derive(Debug, Clone)]
pub struct LinkTag {
    pub rel: String,
    pub href: String,
}

impl Head {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn meta(mut self, name: impl Into<String>, content: impl Into<String>) -> Self {
        self.meta_tags.push(MetaTag {
            name: name.into(),
            content: content.into(),
        });
        self
    }

    pub fn description(self, desc: impl Into<String>) -> Self {
        self.meta("description", desc)
    }

    pub fn keywords(self, keywords: impl Into<String>) -> Self {
        self.meta("keywords", keywords)
    }

    pub fn og_title(self, title: impl Into<String>) -> Self {
        self.meta("og:title", title)
    }

    pub fn og_description(self, desc: impl Into<String>) -> Self {
        self.meta("og:description", desc)
    }

    pub fn og_image(self, url: impl Into<String>) -> Self {
        self.meta("og:image", url)
    }

    pub fn link_stylesheet(mut self, href: impl Into<String>) -> Self {
        self.links.push(LinkTag {
            rel: "stylesheet".to_string(),
            href: href.into(),
        });
        self
    }

    pub fn link(mut self, rel: impl Into<String>, href: impl Into<String>) -> Self {
        self.links.push(LinkTag {
            rel: rel.into(),
            href: href.into(),
        });
        self
    }

    pub fn to_html(&self) -> String {
        let mut parts = Vec::new();
        if let Some(title) = &self.title {
            parts.push(format!("<title>{}</title>", title));
        }
        for meta in &self.meta_tags {
            parts.push(format!(
                "<meta name=\"{}\" content=\"{}\">",
                meta.name, meta.content
            ));
        }
        for link in &self.links {
            parts.push(format!(
                "<link rel=\"{}\" href=\"{}\">",
                link.rel, link.href
            ));
        }
        parts.join("\n    ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_head_title() {
        let head = Head::new().title("My App");
        assert_eq!(head.title, Some("My App".to_string()));
        assert!(head.to_html().contains("<title>My App</title>"));
    }

    #[test]
    fn test_head_meta() {
        let head = Head::new().description("A great app").keywords("rust, web");
        assert_eq!(head.meta_tags.len(), 2);
        let html = head.to_html();
        assert!(html.contains("description"));
        assert!(html.contains("A great app"));
    }

    #[test]
    fn test_head_og() {
        let head = Head::new()
            .og_title("My Page")
            .og_description("Page desc")
            .og_image("https://example.com/img.png");
        assert_eq!(head.meta_tags.len(), 3);
    }

    #[test]
    fn test_head_stylesheet() {
        let head = Head::new().link_stylesheet("/styles.css");
        let html = head.to_html();
        assert!(html.contains("<link rel=\"stylesheet\" href=\"/styles.css\">"));
    }

    #[test]
    fn test_head_empty() {
        let head = Head::new();
        assert_eq!(head.to_html(), "");
    }

    #[test]
    fn test_head_full() {
        let head = Head::new()
            .title("next.rs")
            .description("Rust web framework")
            .link_stylesheet("/app.css");
        let html = head.to_html();
        assert!(html.contains("<title>next.rs</title>"));
        assert!(html.contains("description"));
        assert!(html.contains("stylesheet"));
    }
}
