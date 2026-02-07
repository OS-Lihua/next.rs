use std::path::PathBuf;

use bytes::Bytes;
use http_body_util::Full;
use hyper::{Request, Response, StatusCode};
use next_rs_router::{LayoutResolver, Router};

use crate::api::{ApiRequest, ApiResponse, ApiRouteHandler};
use crate::rsc_handler::RscHandler;
use crate::SsrRenderer;

const RSC_PREFIX: &str = "/_rsc";
const API_PREFIX: &str = "/api";

pub struct RequestHandler {
    router: Router,
    app_dir: PathBuf,
    renderer: SsrRenderer,
    rsc_handler: RscHandler,
    api_handler: ApiRouteHandler,
}

impl RequestHandler {
    pub fn new(router: Router, app_dir: PathBuf) -> Self {
        let renderer = SsrRenderer::new(app_dir.clone());
        let rsc_handler = RscHandler::new(app_dir.clone());
        let api_handler = ApiRouteHandler::new();
        Self {
            router,
            app_dir,
            renderer,
            rsc_handler,
            api_handler,
        }
    }

    pub fn api_handler_mut(&mut self) -> &mut ApiRouteHandler {
        &mut self.api_handler
    }

    pub async fn handle(
        &self,
        req: Request<hyper::body::Incoming>,
    ) -> Result<Response<Full<Bytes>>, hyper::Error> {
        let path = req.uri().path();

        if path.starts_with(RSC_PREFIX) {
            return self.handle_rsc_request(path).await;
        }

        if path.starts_with(API_PREFIX) {
            return self.handle_api_request(&req).await;
        }

        let accepts_rsc = req
            .headers()
            .get("Accept")
            .and_then(|v| v.to_str().ok())
            .map(|v| v.contains("text/x-component"))
            .unwrap_or(false);

        if accepts_rsc {
            return self.handle_rsc_navigation(path).await;
        }

        self.handle_html_request(path).await
    }

    async fn handle_api_request(
        &self,
        req: &Request<hyper::body::Incoming>,
    ) -> Result<Response<Full<Bytes>>, hyper::Error> {
        let path = req.uri().path();

        if let Some(matched) = self.router.match_path(path) {
            if matched.route.is_api() {
                let api_req = ApiRequest::from_hyper(req, matched.params);
                let response = self.api_handler.handle(path, &api_req);
                return Ok(response.into_hyper_response());
            }
        }

        Ok(ApiResponse::not_found("API route not found").into_hyper_response())
    }

    async fn handle_html_request(&self, path: &str) -> Result<Response<Full<Bytes>>, hyper::Error> {
        if let Some(matched) = self.router.match_path(path) {
            let layout_resolver = LayoutResolver::new(&self.app_dir);
            let layout_tree = layout_resolver.resolve(&matched.route);

            let html = self.renderer.render(&layout_tree, &matched.params);

            Ok(Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "text/html; charset=utf-8")
                .body(Full::new(Bytes::from(html)))
                .unwrap())
        } else {
            let html = self.renderer.render_not_found();

            Ok(Response::builder()
                .status(StatusCode::NOT_FOUND)
                .header("Content-Type", "text/html; charset=utf-8")
                .body(Full::new(Bytes::from(html)))
                .unwrap())
        }
    }

    async fn handle_rsc_request(&self, path: &str) -> Result<Response<Full<Bytes>>, hyper::Error> {
        let route_path = path.strip_prefix(RSC_PREFIX).unwrap_or("/");
        let route_path = if route_path.is_empty() {
            "/"
        } else {
            route_path
        };

        if let Some(matched) = self.router.match_path(route_path) {
            let payload = self
                .rsc_handler
                .render_to_wire_format(route_path, &matched.params);

            Ok(Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "text/x-component; charset=utf-8")
                .header("Cache-Control", "no-cache")
                .body(Full::new(Bytes::from(payload)))
                .unwrap())
        } else {
            Ok(Response::builder()
                .status(StatusCode::NOT_FOUND)
                .header("Content-Type", "text/x-component; charset=utf-8")
                .body(Full::new(Bytes::from(
                    "0:{\"type\":\"text\",\"value\":\"404 Not Found\"}",
                )))
                .unwrap())
        }
    }

    async fn handle_rsc_navigation(
        &self,
        path: &str,
    ) -> Result<Response<Full<Bytes>>, hyper::Error> {
        if let Some(matched) = self.router.match_path(path) {
            let payload = self
                .rsc_handler
                .render_to_wire_format(path, &matched.params);

            Ok(Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "text/x-component; charset=utf-8")
                .header("Cache-Control", "no-cache")
                .body(Full::new(Bytes::from(payload)))
                .unwrap())
        } else {
            Ok(Response::builder()
                .status(StatusCode::NOT_FOUND)
                .header("Content-Type", "text/x-component; charset=utf-8")
                .body(Full::new(Bytes::from(
                    "0:{\"type\":\"text\",\"value\":\"404 Not Found\"}",
                )))
                .unwrap())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use next_rs_router::Route;
    use std::fs::{self, File};
    use tempfile::TempDir;

    fn create_test_app() -> (TempDir, PathBuf) {
        let temp = TempDir::new().unwrap();
        let app = temp.path().join("app");

        fs::create_dir_all(&app).unwrap();
        File::create(app.join("page.rs")).unwrap();

        (temp, app)
    }

    #[test]
    fn test_handler_creation() {
        let (_temp, app_dir) = create_test_app();
        let router = Router::from_routes(vec![Route::new("/").with_page(app_dir.join("page.rs"))]);

        let handler = RequestHandler::new(router, app_dir);
        assert_eq!(handler.router.routes.len(), 1);
    }
}
