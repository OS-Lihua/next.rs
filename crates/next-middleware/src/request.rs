use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct NextRequest {
    pub method: String,
    pub url: String,
    pub path: String,
    pub query: HashMap<String, String>,
    pub headers: HashMap<String, String>,
    pub cookies: HashMap<String, String>,
    pub geo: Option<GeoData>,
    pub ip: Option<String>,
}

#[derive(Debug, Clone)]
pub struct GeoData {
    pub country: Option<String>,
    pub region: Option<String>,
    pub city: Option<String>,
}

impl NextRequest {
    pub fn new(method: impl Into<String>, url: impl Into<String>) -> Self {
        let url_str: String = url.into();
        let (path, query) = Self::parse_url(&url_str);

        Self {
            method: method.into(),
            url: url_str,
            path,
            query,
            headers: HashMap::new(),
            cookies: HashMap::new(),
            geo: None,
            ip: None,
        }
    }

    fn parse_url(url: &str) -> (String, HashMap<String, String>) {
        let parts: Vec<&str> = url.splitn(2, '?').collect();
        let path = parts[0].to_string();
        let mut query = HashMap::new();

        if parts.len() > 1 {
            for pair in parts[1].split('&') {
                let kv: Vec<&str> = pair.splitn(2, '=').collect();
                if kv.len() == 2 {
                    query.insert(kv[0].to_string(), kv[1].to_string());
                }
            }
        }

        (path, query)
    }

    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }

    pub fn with_cookie(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.cookies.insert(key.into(), value.into());
        self
    }

    pub fn with_ip(mut self, ip: impl Into<String>) -> Self {
        self.ip = Some(ip.into());
        self
    }

    pub fn header(&self, key: &str) -> Option<&String> {
        self.headers.get(key)
    }

    pub fn cookie(&self, key: &str) -> Option<&String> {
        self.cookies.get(key)
    }

    pub fn query_param(&self, key: &str) -> Option<&String> {
        self.query.get(key)
    }

    pub fn next_url(&self) -> NextUrl {
        NextUrl {
            pathname: self.path.clone(),
            search: if self.query.is_empty() {
                String::new()
            } else {
                format!(
                    "?{}",
                    self.query
                        .iter()
                        .map(|(k, v)| format!("{}={}", k, v))
                        .collect::<Vec<_>>()
                        .join("&")
                )
            },
            origin: self
                .headers
                .get("host")
                .map(|h| format!("https://{}", h))
                .unwrap_or_default(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct NextUrl {
    pub pathname: String,
    pub search: String,
    pub origin: String,
}

impl NextUrl {
    pub fn clone_with_pathname(&self, pathname: impl Into<String>) -> Self {
        Self {
            pathname: pathname.into(),
            search: self.search.clone(),
            origin: self.origin.clone(),
        }
    }

    pub fn href(&self) -> String {
        format!("{}{}{}", self.origin, self.pathname, self.search)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_creation() {
        let req = NextRequest::new("GET", "/api/users?page=1&limit=10");

        assert_eq!(req.method, "GET");
        assert_eq!(req.path, "/api/users");
        assert_eq!(req.query_param("page"), Some(&"1".to_string()));
        assert_eq!(req.query_param("limit"), Some(&"10".to_string()));
    }

    #[test]
    fn test_request_headers_cookies() {
        let req = NextRequest::new("POST", "/login")
            .with_header("Content-Type", "application/json")
            .with_cookie("session", "abc123");

        assert_eq!(
            req.header("Content-Type"),
            Some(&"application/json".to_string())
        );
        assert_eq!(req.cookie("session"), Some(&"abc123".to_string()));
    }

    #[test]
    fn test_next_url() {
        let req = NextRequest::new("GET", "/blog/post?id=123").with_header("host", "example.com");

        let url = req.next_url();
        assert_eq!(url.pathname, "/blog/post");
        assert!(url.search.contains("id=123"));
        assert_eq!(url.origin, "https://example.com");
    }

    #[test]
    fn test_next_url_clone_with_pathname() {
        let url = NextUrl {
            pathname: "/old".to_string(),
            search: "?foo=bar".to_string(),
            origin: "https://test.com".to_string(),
        };

        let new_url = url.clone_with_pathname("/new");
        assert_eq!(new_url.pathname, "/new");
        assert_eq!(new_url.search, "?foo=bar");
    }
}
