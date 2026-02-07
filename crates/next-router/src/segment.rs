#[derive(Debug, Clone, PartialEq)]
pub enum RouteSegment {
    Static(String),
    Dynamic(String),
    CatchAll(String),
    OptionalCatchAll(String),
}

impl RouteSegment {
    pub fn parse(path: &str) -> Vec<RouteSegment> {
        path.split('/')
            .filter(|s| !s.is_empty())
            .map(|segment| {
                if segment.starts_with("[[...") && segment.ends_with("]]") {
                    let name = segment[5..segment.len() - 2].to_string();
                    RouteSegment::OptionalCatchAll(name)
                } else if segment.starts_with("[...") && segment.ends_with(']') {
                    let name = segment[4..segment.len() - 1].to_string();
                    RouteSegment::CatchAll(name)
                } else if segment.starts_with('[') && segment.ends_with(']') {
                    let name = segment[1..segment.len() - 1].to_string();
                    RouteSegment::Dynamic(name)
                } else {
                    RouteSegment::Static(segment.to_string())
                }
            })
            .collect()
    }

    pub fn matches(&self, value: &str) -> bool {
        match self {
            RouteSegment::Static(s) => s == value,
            RouteSegment::Dynamic(_) => !value.is_empty(),
            RouteSegment::CatchAll(_) => true,
            RouteSegment::OptionalCatchAll(_) => true,
        }
    }

    pub fn extract_param(&self, value: &str) -> Option<(String, String)> {
        match self {
            RouteSegment::Static(_) => None,
            RouteSegment::Dynamic(name) => Some((name.clone(), value.to_string())),
            RouteSegment::CatchAll(name) => Some((name.clone(), value.to_string())),
            RouteSegment::OptionalCatchAll(name) => {
                if value.is_empty() {
                    None
                } else {
                    Some((name.clone(), value.to_string()))
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_static() {
        let segments = RouteSegment::parse("/about");
        assert_eq!(segments, vec![RouteSegment::Static("about".to_string())]);
    }

    #[test]
    fn test_parse_dynamic() {
        let segments = RouteSegment::parse("/blog/[slug]");
        assert_eq!(
            segments,
            vec![
                RouteSegment::Static("blog".to_string()),
                RouteSegment::Dynamic("slug".to_string())
            ]
        );
    }

    #[test]
    fn test_parse_catch_all() {
        let segments = RouteSegment::parse("/docs/[...path]");
        assert_eq!(
            segments,
            vec![
                RouteSegment::Static("docs".to_string()),
                RouteSegment::CatchAll("path".to_string())
            ]
        );
    }

    #[test]
    fn test_parse_optional_catch_all() {
        let segments = RouteSegment::parse("/shop/[[...categories]]");
        assert_eq!(
            segments,
            vec![
                RouteSegment::Static("shop".to_string()),
                RouteSegment::OptionalCatchAll("categories".to_string())
            ]
        );
    }

    #[test]
    fn test_segment_matches() {
        assert!(RouteSegment::Static("about".to_string()).matches("about"));
        assert!(!RouteSegment::Static("about".to_string()).matches("contact"));
        assert!(RouteSegment::Dynamic("id".to_string()).matches("123"));
        assert!(RouteSegment::CatchAll("path".to_string()).matches("a/b/c"));
    }

    #[test]
    fn test_extract_param() {
        let segment = RouteSegment::Dynamic("slug".to_string());
        assert_eq!(
            segment.extract_param("hello"),
            Some(("slug".to_string(), "hello".to_string()))
        );

        let static_segment = RouteSegment::Static("about".to_string());
        assert_eq!(static_segment.extract_param("about"), None);
    }
}
