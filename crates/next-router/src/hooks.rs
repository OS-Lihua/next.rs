use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct RouterState {
    pub pathname: String,
    pub params: HashMap<String, String>,
    pub query: HashMap<String, String>,
}

impl RouterState {
    pub fn new(pathname: impl Into<String>) -> Self {
        Self {
            pathname: pathname.into(),
            params: HashMap::new(),
            query: HashMap::new(),
        }
    }

    pub fn with_params(mut self, params: HashMap<String, String>) -> Self {
        self.params = params;
        self
    }

    pub fn with_query(mut self, query: HashMap<String, String>) -> Self {
        self.query = query;
        self
    }

    pub fn param(&self, key: &str) -> Option<&String> {
        self.params.get(key)
    }

    pub fn query_param(&self, key: &str) -> Option<&String> {
        self.query.get(key)
    }
}

pub fn use_router() -> RouterState {
    RouterState::new("/")
}

pub fn use_pathname() -> String {
    "/".to_string()
}

pub fn use_params() -> HashMap<String, String> {
    HashMap::new()
}

pub fn use_search_params() -> HashMap<String, String> {
    HashMap::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_router_state_new() {
        let state = RouterState::new("/blog");
        assert_eq!(state.pathname, "/blog");
        assert!(state.params.is_empty());
        assert!(state.query.is_empty());
    }

    #[test]
    fn test_router_state_with_params() {
        let mut params = HashMap::new();
        params.insert("slug".to_string(), "hello-world".to_string());

        let state = RouterState::new("/blog/hello-world").with_params(params);
        assert_eq!(state.param("slug"), Some(&"hello-world".to_string()));
        assert_eq!(state.param("missing"), None);
    }

    #[test]
    fn test_router_state_with_query() {
        let mut query = HashMap::new();
        query.insert("page".to_string(), "2".to_string());
        query.insert("sort".to_string(), "date".to_string());

        let state = RouterState::new("/blog").with_query(query);
        assert_eq!(state.query_param("page"), Some(&"2".to_string()));
        assert_eq!(state.query_param("sort"), Some(&"date".to_string()));
    }

    #[test]
    fn test_use_router() {
        let state = use_router();
        assert_eq!(state.pathname, "/");
    }

    #[test]
    fn test_use_pathname() {
        let path = use_pathname();
        assert_eq!(path, "/");
    }

    #[test]
    fn test_use_params_empty() {
        let params = use_params();
        assert!(params.is_empty());
    }

    #[test]
    fn test_use_search_params_empty() {
        let params = use_search_params();
        assert!(params.is_empty());
    }
}
