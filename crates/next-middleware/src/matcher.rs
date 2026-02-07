use regex::Regex;

#[derive(Debug, Clone)]
pub enum PathMatcher {
    Exact(String),
    Prefix(String),
    Regex(String),
    All,
}

impl PathMatcher {
    pub fn matches(&self, path: &str) -> bool {
        match self {
            PathMatcher::Exact(p) => path == p,
            PathMatcher::Prefix(p) => path.starts_with(p),
            PathMatcher::Regex(pattern) => Regex::new(pattern)
                .map(|re| re.is_match(path))
                .unwrap_or(false),
            PathMatcher::All => true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct MiddlewareMatcher {
    include: Vec<PathMatcher>,
    exclude: Vec<PathMatcher>,
}

impl MiddlewareMatcher {
    pub fn new() -> Self {
        Self {
            include: Vec::new(),
            exclude: Vec::new(),
        }
    }

    pub fn include(mut self, matcher: PathMatcher) -> Self {
        self.include.push(matcher);
        self
    }

    pub fn exclude(mut self, matcher: PathMatcher) -> Self {
        self.exclude.push(matcher);
        self
    }

    pub fn matches(&self, path: &str) -> bool {
        for excluded in &self.exclude {
            if excluded.matches(path) {
                return false;
            }
        }

        if self.include.is_empty() {
            return true;
        }

        for included in &self.include {
            if included.matches(path) {
                return true;
            }
        }

        false
    }

    pub fn from_config(patterns: Vec<&str>) -> Self {
        let mut matcher = Self::new();
        for pattern in patterns {
            if pattern.ends_with("*") {
                matcher.include.push(PathMatcher::Prefix(
                    pattern.trim_end_matches('*').to_string(),
                ));
            } else if pattern.starts_with("^") || pattern.contains("(") {
                matcher
                    .include
                    .push(PathMatcher::Regex(pattern.to_string()));
            } else {
                matcher
                    .include
                    .push(PathMatcher::Exact(pattern.to_string()));
            }
        }
        matcher
    }
}

impl Default for MiddlewareMatcher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_matcher() {
        let matcher = PathMatcher::Exact("/about".to_string());
        assert!(matcher.matches("/about"));
        assert!(!matcher.matches("/about/"));
        assert!(!matcher.matches("/about/team"));
    }

    #[test]
    fn test_prefix_matcher() {
        let matcher = PathMatcher::Prefix("/api/".to_string());
        assert!(matcher.matches("/api/users"));
        assert!(matcher.matches("/api/posts/1"));
        assert!(!matcher.matches("/about"));
    }

    #[test]
    fn test_regex_matcher() {
        let matcher = PathMatcher::Regex(r"^/blog/\d+$".to_string());
        assert!(matcher.matches("/blog/123"));
        assert!(!matcher.matches("/blog/abc"));
    }

    #[test]
    fn test_middleware_matcher_include() {
        let matcher = MiddlewareMatcher::new()
            .include(PathMatcher::Prefix("/api/".to_string()))
            .include(PathMatcher::Prefix("/admin/".to_string()));

        assert!(matcher.matches("/api/users"));
        assert!(matcher.matches("/admin/dashboard"));
        assert!(!matcher.matches("/public/file"));
    }

    #[test]
    fn test_middleware_matcher_exclude() {
        let matcher = MiddlewareMatcher::new()
            .include(PathMatcher::All)
            .exclude(PathMatcher::Prefix("/static/".to_string()))
            .exclude(PathMatcher::Prefix("/_next/".to_string()));

        assert!(matcher.matches("/api/users"));
        assert!(!matcher.matches("/static/image.png"));
        assert!(!matcher.matches("/_next/chunk.js"));
    }

    #[test]
    fn test_from_config() {
        let matcher = MiddlewareMatcher::from_config(vec!["/api/*", "/admin/*", "/login"]);

        assert!(matcher.matches("/api/users"));
        assert!(matcher.matches("/admin/"));
        assert!(matcher.matches("/login"));
        assert!(!matcher.matches("/public"));
    }
}
