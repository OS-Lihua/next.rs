mod boundary;
pub mod codegen;
mod hooks;
mod layout;
mod link;
mod matcher;
mod scanner;
mod segment;

pub use boundary::{
    BoundaryResolver, BoundaryStack, ErrorBoundary, LoadingBoundary, NotFoundBoundary,
};
pub use codegen::RouteCodegen;
pub use hooks::{use_params, use_pathname, use_router, use_search_params, RouterState};
pub use layout::{LayoutResolver, RouteMetadata};
pub use link::{link, Link};
pub use matcher::{MatchedRoute, RouteMatcher};
pub use scanner::{RouteScanner, SpecialFile};
pub use segment::RouteSegment;

use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Route {
    pub path: String,
    pub segments: Vec<RouteSegment>,
    pub page_file: Option<PathBuf>,
    pub layout_file: Option<PathBuf>,
    pub loading_file: Option<PathBuf>,
    pub error_file: Option<PathBuf>,
    pub not_found_file: Option<PathBuf>,
    pub route_file: Option<PathBuf>,
}

impl Route {
    pub fn new(path: impl Into<String>) -> Self {
        let path = path.into();
        let segments = RouteSegment::parse(&path);
        Self {
            path,
            segments,
            page_file: None,
            layout_file: None,
            loading_file: None,
            error_file: None,
            not_found_file: None,
            route_file: None,
        }
    }

    pub fn with_page(mut self, file: PathBuf) -> Self {
        self.page_file = Some(file);
        self
    }

    pub fn with_layout(mut self, file: PathBuf) -> Self {
        self.layout_file = Some(file);
        self
    }

    pub fn is_dynamic(&self) -> bool {
        self.segments
            .iter()
            .any(|s| !matches!(s, RouteSegment::Static(_)))
    }

    pub fn is_api(&self) -> bool {
        self.route_file.is_some()
    }
}

#[derive(Debug, Clone)]
pub struct Router {
    pub routes: Vec<Route>,
}

impl Router {
    pub fn new() -> Self {
        Self { routes: Vec::new() }
    }

    pub fn from_routes(routes: Vec<Route>) -> Self {
        Self { routes }
    }

    pub fn add_route(&mut self, route: Route) {
        self.routes.push(route);
    }

    pub fn match_path(&self, path: &str) -> Option<MatchedRoute> {
        let matcher = RouteMatcher::new(&self.routes);
        matcher.match_path(path)
    }

    pub fn static_routes(&self) -> impl Iterator<Item = &Route> {
        self.routes.iter().filter(|r| !r.is_dynamic())
    }

    pub fn dynamic_routes(&self) -> impl Iterator<Item = &Route> {
        self.routes.iter().filter(|r| r.is_dynamic())
    }
}

impl Default for Router {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct Layout {
    pub file: PathBuf,
    pub path: String,
}

#[derive(Debug)]
pub struct LayoutTree {
    pub layouts: Vec<Layout>,
    pub page: PathBuf,
}

impl LayoutTree {
    pub fn new(page: PathBuf) -> Self {
        Self {
            layouts: Vec::new(),
            page,
        }
    }

    pub fn add_layout(&mut self, layout: Layout) {
        self.layouts.push(layout);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_route_creation() {
        let route = Route::new("/blog/[slug]");
        assert_eq!(route.path, "/blog/[slug]");
        assert!(route.is_dynamic());
    }

    #[test]
    fn test_static_route() {
        let route = Route::new("/about");
        assert!(!route.is_dynamic());
    }

    #[test]
    fn test_router() {
        let mut router = Router::new();
        router.add_route(Route::new("/"));
        router.add_route(Route::new("/about"));
        router.add_route(Route::new("/blog/[slug]"));

        assert_eq!(router.routes.len(), 3);
        assert_eq!(router.static_routes().count(), 2);
        assert_eq!(router.dynamic_routes().count(), 1);
    }
}
