use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum MiddlewareResult {
    Next,
    Rewrite(String),
    Redirect(RedirectResponse),
    Response(NextResponse),
}

#[derive(Debug, Clone)]
pub struct RedirectResponse {
    pub url: String,
    pub status: u16,
    pub headers: HashMap<String, String>,
}

impl RedirectResponse {
    pub fn temporary(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            status: 307,
            headers: HashMap::new(),
        }
    }

    pub fn permanent(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            status: 308,
            headers: HashMap::new(),
        }
    }

    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }
}

#[derive(Debug, Clone)]
pub struct NextResponse {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub cookies: Vec<SetCookie>,
    pub body: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SetCookie {
    pub name: String,
    pub value: String,
    pub path: Option<String>,
    pub domain: Option<String>,
    pub max_age: Option<i64>,
    pub http_only: bool,
    pub secure: bool,
    pub same_site: Option<SameSite>,
}

#[derive(Debug, Clone, Copy)]
pub enum SameSite {
    Strict,
    Lax,
    None,
}

impl NextResponse {
    pub fn next() -> MiddlewareResult {
        MiddlewareResult::Next
    }

    pub fn redirect(url: impl Into<String>) -> MiddlewareResult {
        MiddlewareResult::Redirect(RedirectResponse::temporary(url))
    }

    pub fn redirect_permanent(url: impl Into<String>) -> MiddlewareResult {
        MiddlewareResult::Redirect(RedirectResponse::permanent(url))
    }

    pub fn rewrite(url: impl Into<String>) -> MiddlewareResult {
        MiddlewareResult::Rewrite(url.into())
    }

    pub fn new(status: u16) -> Self {
        Self {
            status,
            headers: HashMap::new(),
            cookies: Vec::new(),
            body: None,
        }
    }

    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }

    pub fn with_body(mut self, body: impl Into<String>) -> Self {
        self.body = Some(body.into());
        self
    }

    pub fn set_cookie(mut self, cookie: SetCookie) -> Self {
        self.cookies.push(cookie);
        self
    }

    pub fn into_result(self) -> MiddlewareResult {
        MiddlewareResult::Response(self)
    }
}

impl SetCookie {
    pub fn new(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
            path: None,
            domain: None,
            max_age: None,
            http_only: false,
            secure: false,
            same_site: None,
        }
    }

    pub fn with_path(mut self, path: impl Into<String>) -> Self {
        self.path = Some(path.into());
        self
    }

    pub fn with_max_age(mut self, seconds: i64) -> Self {
        self.max_age = Some(seconds);
        self
    }

    pub fn http_only(mut self) -> Self {
        self.http_only = true;
        self
    }

    pub fn secure(mut self) -> Self {
        self.secure = true;
        self
    }

    pub fn with_same_site(mut self, same_site: SameSite) -> Self {
        self.same_site = Some(same_site);
        self
    }

    pub fn to_header_value(&self) -> String {
        let mut parts = vec![format!("{}={}", self.name, self.value)];

        if let Some(path) = &self.path {
            parts.push(format!("Path={}", path));
        }
        if let Some(domain) = &self.domain {
            parts.push(format!("Domain={}", domain));
        }
        if let Some(max_age) = self.max_age {
            parts.push(format!("Max-Age={}", max_age));
        }
        if self.http_only {
            parts.push("HttpOnly".to_string());
        }
        if self.secure {
            parts.push("Secure".to_string());
        }
        if let Some(same_site) = &self.same_site {
            parts.push(format!(
                "SameSite={}",
                match same_site {
                    SameSite::Strict => "Strict",
                    SameSite::Lax => "Lax",
                    SameSite::None => "None",
                }
            ));
        }

        parts.join("; ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_next_response_redirect() {
        let result = NextResponse::redirect("/login");
        if let MiddlewareResult::Redirect(redirect) = result {
            assert_eq!(redirect.url, "/login");
            assert_eq!(redirect.status, 307);
        } else {
            panic!("Expected Redirect");
        }
    }

    #[test]
    fn test_next_response_rewrite() {
        let result = NextResponse::rewrite("/api/v2/users");
        if let MiddlewareResult::Rewrite(url) = result {
            assert_eq!(url, "/api/v2/users");
        } else {
            panic!("Expected Rewrite");
        }
    }

    #[test]
    fn test_response_with_headers() {
        let response = NextResponse::new(200)
            .with_header("X-Custom", "value")
            .with_body("Hello");

        assert_eq!(response.status, 200);
        assert_eq!(response.headers.get("X-Custom"), Some(&"value".to_string()));
        assert_eq!(response.body, Some("Hello".to_string()));
    }

    #[test]
    fn test_set_cookie() {
        let cookie = SetCookie::new("session", "abc123")
            .with_path("/")
            .with_max_age(3600)
            .http_only()
            .secure()
            .with_same_site(SameSite::Strict);

        let header = cookie.to_header_value();
        assert!(header.contains("session=abc123"));
        assert!(header.contains("Path=/"));
        assert!(header.contains("Max-Age=3600"));
        assert!(header.contains("HttpOnly"));
        assert!(header.contains("Secure"));
        assert!(header.contains("SameSite=Strict"));
    }

    #[test]
    fn test_response_with_cookie() {
        let response = NextResponse::new(200).set_cookie(SetCookie::new("token", "xyz"));

        assert_eq!(response.cookies.len(), 1);
        assert_eq!(response.cookies[0].name, "token");
    }
}
