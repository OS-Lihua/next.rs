use std::collections::HashMap;

use crate::segment::RouteSegment;
use crate::Route;

#[derive(Debug, Clone)]
pub struct MatchedRoute {
    pub route: Route,
    pub params: HashMap<String, String>,
}

pub struct RouteMatcher<'a> {
    routes: &'a [Route],
}

impl<'a> RouteMatcher<'a> {
    pub fn new(routes: &'a [Route]) -> Self {
        Self { routes }
    }

    pub fn match_path(&self, path: &str) -> Option<MatchedRoute> {
        let path_segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();

        let mut best_match: Option<(MatchedRoute, u32)> = None;

        for route in self.routes {
            if let Some((params, priority)) = self.try_match(route, &path_segments) {
                let matched = MatchedRoute {
                    route: route.clone(),
                    params,
                };

                match &best_match {
                    Some((_, best_priority)) if priority <= *best_priority => {}
                    _ => best_match = Some((matched, priority)),
                }
            }
        }

        best_match.map(|(m, _)| m)
    }

    fn try_match(
        &self,
        route: &Route,
        path_segments: &[&str],
    ) -> Option<(HashMap<String, String>, u32)> {
        let route_segments = &route.segments;
        let mut params = HashMap::new();
        let mut priority = 0u32;
        let mut path_idx = 0;

        for segment in route_segments.iter() {
            match segment {
                RouteSegment::Static(expected) => {
                    if path_idx >= path_segments.len() {
                        return None;
                    }
                    if path_segments[path_idx] != expected {
                        return None;
                    }
                    priority += 1000;
                    path_idx += 1;
                }
                RouteSegment::Dynamic(name) => {
                    if path_idx >= path_segments.len() {
                        return None;
                    }
                    params.insert(name.clone(), path_segments[path_idx].to_string());
                    priority += 100;
                    path_idx += 1;
                }
                RouteSegment::CatchAll(name) => {
                    if path_idx >= path_segments.len() {
                        return None;
                    }
                    let remaining: Vec<&str> = path_segments[path_idx..].to_vec();
                    params.insert(name.clone(), remaining.join("/"));
                    priority += 10;
                    path_idx = path_segments.len();
                }
                RouteSegment::OptionalCatchAll(name) => {
                    if path_idx < path_segments.len() {
                        let remaining: Vec<&str> = path_segments[path_idx..].to_vec();
                        params.insert(name.clone(), remaining.join("/"));
                    }
                    priority += 1;
                    path_idx = path_segments.len();
                }
            }
        }

        if path_idx == path_segments.len() {
            Some((params, priority))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_match_static_route() {
        let routes = vec![Route::new("/about"), Route::new("/contact")];
        let matcher = RouteMatcher::new(&routes);

        let result = matcher.match_path("/about");
        assert!(result.is_some());
        assert_eq!(result.unwrap().route.path, "/about");

        let result = matcher.match_path("/contact");
        assert!(result.is_some());
        assert_eq!(result.unwrap().route.path, "/contact");

        let result = matcher.match_path("/unknown");
        assert!(result.is_none());
    }

    #[test]
    fn test_match_dynamic_route() {
        let routes = vec![Route::new("/blog/[slug]")];
        let matcher = RouteMatcher::new(&routes);

        let result = matcher.match_path("/blog/hello-world");
        assert!(result.is_some());
        let matched = result.unwrap();
        assert_eq!(matched.route.path, "/blog/[slug]");
        assert_eq!(matched.params.get("slug"), Some(&"hello-world".to_string()));
    }

    #[test]
    fn test_match_catch_all_route() {
        let routes = vec![Route::new("/docs/[...path]")];
        let matcher = RouteMatcher::new(&routes);

        let result = matcher.match_path("/docs/getting-started/installation");
        assert!(result.is_some());
        let matched = result.unwrap();
        assert_eq!(matched.route.path, "/docs/[...path]");
        assert_eq!(
            matched.params.get("path"),
            Some(&"getting-started/installation".to_string())
        );
    }

    #[test]
    fn test_match_optional_catch_all() {
        let routes = vec![Route::new("/shop/[[...categories]]")];
        let matcher = RouteMatcher::new(&routes);

        let result = matcher.match_path("/shop");
        assert!(result.is_some());
        let matched = result.unwrap();
        assert!(!matched.params.contains_key("categories"));

        let result = matcher.match_path("/shop/electronics/phones");
        assert!(result.is_some());
        let matched = result.unwrap();
        assert_eq!(
            matched.params.get("categories"),
            Some(&"electronics/phones".to_string())
        );
    }

    #[test]
    fn test_static_takes_priority_over_dynamic() {
        let routes = vec![Route::new("/blog/[slug]"), Route::new("/blog/featured")];
        let matcher = RouteMatcher::new(&routes);

        let result = matcher.match_path("/blog/featured");
        assert!(result.is_some());
        assert_eq!(result.unwrap().route.path, "/blog/featured");

        let result = matcher.match_path("/blog/other-post");
        assert!(result.is_some());
        assert_eq!(result.unwrap().route.path, "/blog/[slug]");
    }

    #[test]
    fn test_dynamic_takes_priority_over_catch_all() {
        let routes = vec![Route::new("/api/[...path]"), Route::new("/api/[endpoint]")];
        let matcher = RouteMatcher::new(&routes);

        let result = matcher.match_path("/api/users");
        assert!(result.is_some());
        assert_eq!(result.unwrap().route.path, "/api/[endpoint]");

        let result = matcher.match_path("/api/users/123/profile");
        assert!(result.is_some());
        assert_eq!(result.unwrap().route.path, "/api/[...path]");
    }

    #[test]
    fn test_root_route() {
        let routes = vec![Route::new("/")];
        let matcher = RouteMatcher::new(&routes);

        let result = matcher.match_path("/");
        assert!(result.is_some());
        assert_eq!(result.unwrap().route.path, "/");
    }

    #[test]
    fn test_multiple_dynamic_segments() {
        let routes = vec![Route::new("/[category]/[product]")];
        let matcher = RouteMatcher::new(&routes);

        let result = matcher.match_path("/electronics/laptop");
        assert!(result.is_some());
        let matched = result.unwrap();
        assert_eq!(
            matched.params.get("category"),
            Some(&"electronics".to_string())
        );
        assert_eq!(matched.params.get("product"), Some(&"laptop".to_string()));
    }
}
