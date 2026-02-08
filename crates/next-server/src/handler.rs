use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use bytes::Bytes;
use http_body_util::Full;
use hyper::{Request, Response, StatusCode};
use next_rs_router::Router;

use crate::api::{ApiRequest, ApiResponse, ApiRouteHandler};
use crate::rsc_handler::RscHandler;
use crate::ssr::{PageRegistry, SsrRenderer};

const RSC_PREFIX: &str = "/_rsc";
const API_PREFIX: &str = "/api";
const ACTION_PREFIX: &str = "/_action/";
const WS_PREFIX: &str = "/ws/";

pub struct RequestHandler {
    router: Router,
    #[allow(dead_code)]
    app_dir: PathBuf,
    renderer: SsrRenderer,
    registry: Arc<PageRegistry>,
    rsc_handler: RscHandler,
    api_handler: ApiRouteHandler,
    action_registry: Arc<next_rs_actions::ActionRegistry>,
    ws_registry: Arc<crate::ws::WsRegistry>,
}

impl RequestHandler {
    pub fn new(router: Router, app_dir: PathBuf, registry: Arc<PageRegistry>) -> Self {
        let renderer = SsrRenderer::new();
        let rsc_handler = RscHandler::new(app_dir.clone());
        let api_handler = ApiRouteHandler::new();
        let action_registry = Arc::new(next_rs_actions::ActionRegistry::new());
        let ws_registry = Arc::new(crate::ws::WsRegistry::new());
        Self {
            router,
            app_dir,
            renderer,
            registry,
            rsc_handler,
            api_handler,
            action_registry,
            ws_registry,
        }
    }

    pub fn with_ws_registry(mut self, registry: crate::ws::WsRegistry) -> Self {
        self.ws_registry = Arc::new(registry);
        self
    }

    pub fn action_registry(&self) -> &Arc<next_rs_actions::ActionRegistry> {
        &self.action_registry
    }

    pub fn api_handler_mut(&mut self) -> &mut ApiRouteHandler {
        &mut self.api_handler
    }

    pub async fn handle(
        &self,
        req: Request<hyper::body::Incoming>,
    ) -> Result<Response<Full<Bytes>>, hyper::Error> {
        let path = req.uri().path().to_string();

        if let Some(response) = self.try_serve_static(&path).await {
            return Ok(response);
        }

        if path.starts_with(WS_PREFIX) || path == "/ws" {
            if let Some(handler_fn) = self.ws_registry.get_handler(&path) {
                return crate::ws::handle_ws_upgrade(req, handler_fn.clone()).await;
            }
        }

        if path.starts_with(ACTION_PREFIX) {
            return self.handle_action_request(&path, req).await;
        }

        if path.starts_with(RSC_PREFIX) {
            return self.handle_rsc_request(&path).await;
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
            return self.handle_rsc_navigation(&path).await;
        }

        self.handle_html_request(&path).await
    }

    async fn handle_action_request(
        &self,
        path: &str,
        req: Request<hyper::body::Incoming>,
    ) -> Result<Response<Full<Bytes>>, hyper::Error> {
        let action_id = path.strip_prefix(ACTION_PREFIX).unwrap_or("");

        let body_bytes = match http_body_util::BodyExt::collect(req.into_body()).await {
            Ok(collected) => collected.to_bytes(),
            Err(_) => {
                let resp = next_rs_actions::ActionResponse::error(
                    next_rs_actions::ActionError::new("Failed to read request body"),
                );
                let json = serde_json::to_string(&resp).unwrap_or_default();
                return Ok(Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .header("Content-Type", "application/json")
                    .body(Full::new(Bytes::from(json)))
                    .unwrap());
            }
        };

        let payload: serde_json::Value =
            serde_json::from_slice(&body_bytes).unwrap_or(serde_json::Value::Null);

        let request = next_rs_actions::ActionRequest {
            action_id: action_id.to_string(),
            payload,
        };

        let response = self.action_registry.execute(request).await;
        let status = if response.success {
            StatusCode::OK
        } else {
            StatusCode::BAD_REQUEST
        };
        let json = serde_json::to_string(&response).unwrap_or_default();

        Ok(Response::builder()
            .status(status)
            .header("Content-Type", "application/json")
            .body(Full::new(Bytes::from(json)))
            .unwrap())
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

    async fn try_serve_static(&self, path: &str) -> Option<Response<Full<Bytes>>> {
        if path == "/" || !path.contains('.') {
            return None;
        }

        let clean = path.trim_start_matches('/');

        let candidates = [
            PathBuf::from("public").join(clean),
            PathBuf::from(".next/static").join(clean),
            PathBuf::from("pkg").join(clean),
        ];

        for file_path in candidates {
            if file_path.exists() && file_path.is_file() {
                if let Ok(content) = fs::read(&file_path) {
                    let content_type = match file_path.extension().and_then(|e| e.to_str()) {
                        Some("html") => "text/html; charset=utf-8",
                        Some("js") => "application/javascript",
                        Some("mjs") => "application/javascript",
                        Some("wasm") => "application/wasm",
                        Some("css") => "text/css",
                        Some("json") => "application/json",
                        Some("png") => "image/png",
                        Some("jpg") | Some("jpeg") => "image/jpeg",
                        Some("gif") => "image/gif",
                        Some("svg") => "image/svg+xml",
                        Some("ico") => "image/x-icon",
                        Some("woff2") => "font/woff2",
                        Some("woff") => "font/woff",
                        Some("ttf") => "font/ttf",
                        _ => "application/octet-stream",
                    };

                    return Some(
                        Response::builder()
                            .status(StatusCode::OK)
                            .header("Content-Type", content_type)
                            .header("Cache-Control", "public, max-age=31536000, immutable")
                            .body(Full::new(Bytes::from(content)))
                            .unwrap(),
                    );
                }
            }
        }

        None
    }

    async fn handle_html_request(&self, path: &str) -> Result<Response<Full<Bytes>>, hyper::Error> {
        if let Some(matched) = self.router.match_path(path) {
            let html = self
                .renderer
                .render(&matched.route.path, &matched.params, &self.registry);

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
        let registry = Arc::new(PageRegistry::new());

        let handler = RequestHandler::new(router, app_dir, registry);
        assert_eq!(handler.router.routes.len(), 1);
    }
}
