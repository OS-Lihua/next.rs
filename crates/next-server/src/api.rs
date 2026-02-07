use bytes::Bytes;
use http_body_util::Full;
use hyper::{Method, Request, Response, StatusCode};
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ApiRequest {
    pub method: Method,
    pub path: String,
    pub params: HashMap<String, String>,
    pub query: HashMap<String, String>,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
}

impl ApiRequest {
    pub fn from_hyper(
        req: &Request<hyper::body::Incoming>,
        params: HashMap<String, String>,
    ) -> Self {
        let path = req.uri().path().to_string();
        let query = req
            .uri()
            .query()
            .map(parse_query_string)
            .unwrap_or_default();

        let headers = req
            .headers()
            .iter()
            .filter_map(|(k, v)| {
                v.to_str()
                    .ok()
                    .map(|v| (k.as_str().to_string(), v.to_string()))
            })
            .collect();

        Self {
            method: req.method().clone(),
            path,
            params,
            query,
            headers,
            body: None,
        }
    }

    pub fn param(&self, key: &str) -> Option<&str> {
        self.params.get(key).map(|s| s.as_str())
    }

    pub fn query_param(&self, key: &str) -> Option<&str> {
        self.query.get(key).map(|s| s.as_str())
    }

    pub fn header(&self, key: &str) -> Option<&str> {
        self.headers.get(key).map(|s| s.as_str())
    }
}

fn parse_query_string(query: &str) -> HashMap<String, String> {
    query
        .split('&')
        .filter_map(|pair| {
            let mut parts = pair.splitn(2, '=');
            let key = parts.next()?;
            let value = parts.next().unwrap_or("");
            Some((key.to_string(), value.to_string()))
        })
        .collect()
}

#[derive(Debug, Clone)]
pub struct ApiResponse {
    pub status: StatusCode,
    pub headers: HashMap<String, String>,
    pub body: String,
}

impl ApiResponse {
    pub fn new(status: StatusCode, body: impl Into<String>) -> Self {
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        Self {
            status,
            headers,
            body: body.into(),
        }
    }

    pub fn ok() -> Self {
        Self::new(StatusCode::OK, "{}")
    }

    pub fn json<T: Serialize>(data: &T) -> Self {
        let body = serde_json::to_string(data).unwrap_or_else(|_| "{}".to_string());
        Self::new(StatusCode::OK, body)
    }

    pub fn created<T: Serialize>(data: &T) -> Self {
        let body = serde_json::to_string(data).unwrap_or_else(|_| "{}".to_string());
        Self::new(StatusCode::CREATED, body)
    }

    pub fn no_content() -> Self {
        Self::new(StatusCode::NO_CONTENT, "")
    }

    pub fn bad_request(message: &str) -> Self {
        let body = serde_json::json!({"error": message}).to_string();
        Self::new(StatusCode::BAD_REQUEST, body)
    }

    pub fn not_found(message: &str) -> Self {
        let body = serde_json::json!({"error": message}).to_string();
        Self::new(StatusCode::NOT_FOUND, body)
    }

    pub fn internal_error(message: &str) -> Self {
        let body = serde_json::json!({"error": message}).to_string();
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, body)
    }

    pub fn method_not_allowed() -> Self {
        let body = serde_json::json!({"error": "Method not allowed"}).to_string();
        Self::new(StatusCode::METHOD_NOT_ALLOWED, body)
    }

    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }

    pub fn with_status(mut self, status: StatusCode) -> Self {
        self.status = status;
        self
    }

    pub fn into_hyper_response(self) -> Response<Full<Bytes>> {
        let mut builder = Response::builder().status(self.status);

        for (key, value) in &self.headers {
            builder = builder.header(key.as_str(), value.as_str());
        }

        builder.body(Full::new(Bytes::from(self.body))).unwrap()
    }
}

impl Default for ApiResponse {
    fn default() -> Self {
        Self::ok()
    }
}

pub type ApiHandler = Box<dyn Fn(&ApiRequest) -> ApiResponse + Send + Sync>;

pub struct ApiRouteHandler {
    handlers: HashMap<String, RouteHandlers>,
}

struct RouteHandlers {
    get: Option<ApiHandler>,
    post: Option<ApiHandler>,
    put: Option<ApiHandler>,
    patch: Option<ApiHandler>,
    delete: Option<ApiHandler>,
    head: Option<ApiHandler>,
    options: Option<ApiHandler>,
}

impl RouteHandlers {
    fn new() -> Self {
        Self {
            get: None,
            post: None,
            put: None,
            patch: None,
            delete: None,
            head: None,
            options: None,
        }
    }

    fn handle(&self, method: &Method, req: &ApiRequest) -> ApiResponse {
        let handler = match *method {
            Method::GET => &self.get,
            Method::POST => &self.post,
            Method::PUT => &self.put,
            Method::PATCH => &self.patch,
            Method::DELETE => &self.delete,
            Method::HEAD => &self.head,
            Method::OPTIONS => &self.options,
            _ => &None,
        };

        match handler {
            Some(h) => h(req),
            None => {
                if *method == Method::OPTIONS {
                    self.handle_options()
                } else {
                    ApiResponse::method_not_allowed()
                }
            }
        }
    }

    fn handle_options(&self) -> ApiResponse {
        let mut methods = vec!["OPTIONS"];
        if self.get.is_some() {
            methods.push("GET");
        }
        if self.post.is_some() {
            methods.push("POST");
        }
        if self.put.is_some() {
            methods.push("PUT");
        }
        if self.patch.is_some() {
            methods.push("PATCH");
        }
        if self.delete.is_some() {
            methods.push("DELETE");
        }
        if self.head.is_some() {
            methods.push("HEAD");
        }

        ApiResponse::ok()
            .with_header("Allow", methods.join(", "))
            .with_header("Access-Control-Allow-Methods", methods.join(", "))
    }
}

impl ApiRouteHandler {
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }

    pub fn register_get<F>(&mut self, path: &str, handler: F)
    where
        F: Fn(&ApiRequest) -> ApiResponse + Send + Sync + 'static,
    {
        let entry = self
            .handlers
            .entry(path.to_string())
            .or_insert_with(RouteHandlers::new);
        entry.get = Some(Box::new(handler));
    }

    pub fn register_post<F>(&mut self, path: &str, handler: F)
    where
        F: Fn(&ApiRequest) -> ApiResponse + Send + Sync + 'static,
    {
        let entry = self
            .handlers
            .entry(path.to_string())
            .or_insert_with(RouteHandlers::new);
        entry.post = Some(Box::new(handler));
    }

    pub fn register_put<F>(&mut self, path: &str, handler: F)
    where
        F: Fn(&ApiRequest) -> ApiResponse + Send + Sync + 'static,
    {
        let entry = self
            .handlers
            .entry(path.to_string())
            .or_insert_with(RouteHandlers::new);
        entry.put = Some(Box::new(handler));
    }

    pub fn register_delete<F>(&mut self, path: &str, handler: F)
    where
        F: Fn(&ApiRequest) -> ApiResponse + Send + Sync + 'static,
    {
        let entry = self
            .handlers
            .entry(path.to_string())
            .or_insert_with(RouteHandlers::new);
        entry.delete = Some(Box::new(handler));
    }

    pub fn handle(&self, path: &str, req: &ApiRequest) -> ApiResponse {
        if let Some(handlers) = self.handlers.get(path) {
            handlers.handle(&req.method, req)
        } else {
            ApiResponse::not_found("API route not found")
        }
    }

    pub fn has_route(&self, path: &str) -> bool {
        self.handlers.contains_key(path)
    }
}

impl Default for ApiRouteHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_response_json() {
        #[derive(Serialize)]
        struct User {
            id: i32,
            name: String,
        }

        let user = User {
            id: 1,
            name: "Alice".to_string(),
        };
        let response = ApiResponse::json(&user);

        assert_eq!(response.status, StatusCode::OK);
        assert!(response.body.contains("Alice"));
    }

    #[test]
    fn test_api_response_not_found() {
        let response = ApiResponse::not_found("User not found");
        assert_eq!(response.status, StatusCode::NOT_FOUND);
        assert!(response.body.contains("User not found"));
    }

    #[test]
    fn test_api_response_with_header() {
        let response = ApiResponse::ok()
            .with_header("X-Custom", "value")
            .with_header("Cache-Control", "no-cache");

        assert_eq!(response.headers.get("X-Custom"), Some(&"value".to_string()));
    }

    #[test]
    fn test_parse_query_string() {
        let query = parse_query_string("foo=bar&baz=qux&empty=");
        assert_eq!(query.get("foo"), Some(&"bar".to_string()));
        assert_eq!(query.get("baz"), Some(&"qux".to_string()));
        assert_eq!(query.get("empty"), Some(&"".to_string()));
    }

    #[test]
    fn test_api_route_handler() {
        let mut handler = ApiRouteHandler::new();

        handler.register_get("/api/users", |_req| {
            ApiResponse::json(&vec!["user1", "user2"])
        });

        handler.register_post("/api/users", |_req| {
            ApiResponse::created(&serde_json::json!({"id": 1}))
        });

        assert!(handler.has_route("/api/users"));
        assert!(!handler.has_route("/api/posts"));
    }

    #[test]
    fn test_api_request_params() {
        let mut params = HashMap::new();
        params.insert("id".to_string(), "123".to_string());

        let mut query = HashMap::new();
        query.insert("page".to_string(), "1".to_string());

        let req = ApiRequest {
            method: Method::GET,
            path: "/api/users/123".to_string(),
            params,
            query,
            headers: HashMap::new(),
            body: None,
        };

        assert_eq!(req.param("id"), Some("123"));
        assert_eq!(req.query_param("page"), Some("1"));
    }

    #[test]
    fn test_method_not_allowed() {
        let mut handler = ApiRouteHandler::new();
        handler.register_get("/api/readonly", |_| ApiResponse::ok());

        let req = ApiRequest {
            method: Method::POST,
            path: "/api/readonly".to_string(),
            params: HashMap::new(),
            query: HashMap::new(),
            headers: HashMap::new(),
            body: None,
        };

        let response = handler.handle("/api/readonly", &req);
        assert_eq!(response.status, StatusCode::METHOD_NOT_ALLOWED);
    }

    #[test]
    fn test_options_response() {
        let mut handler = ApiRouteHandler::new();
        handler.register_get("/api/users", |_| ApiResponse::ok());
        handler.register_post("/api/users", |_| ApiResponse::ok());

        let req = ApiRequest {
            method: Method::OPTIONS,
            path: "/api/users".to_string(),
            params: HashMap::new(),
            query: HashMap::new(),
            headers: HashMap::new(),
            body: None,
        };

        let response = handler.handle("/api/users", &req);
        assert_eq!(response.status, StatusCode::OK);
        assert!(response.headers.get("Allow").unwrap().contains("GET"));
        assert!(response.headers.get("Allow").unwrap().contains("POST"));
    }
}
