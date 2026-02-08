mod api;
mod handler;
mod isr;
mod rsc_handler;
mod ssg;
mod ssr;
mod streaming;
pub mod ws;

pub use api::{ApiRequest, ApiResponse, ApiRouteHandler};
pub use handler::RequestHandler;
pub use isr::{CacheEntry, IncrementalCache, IsrConfig};
pub use rsc_handler::RscHandler;
pub use ssg::{GeneratedFile, GenerationResult, StaticGenerator, StaticParams};
pub use ssr::{LayoutRenderFn, PageRegistry, PageRenderFn, SsrRenderer};
pub use streaming::{HtmlStream, RscStream, RscStreamingRenderer, StreamingRenderer};

pub use next_rs_actions::ActionRegistry;
pub use ws::{WsConnection, WsMessage, WsReceiver, WsRegistry, WsSender};

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use next_rs_router::{RouteScanner, Router};
use tokio::net::TcpListener;

pub struct ServerConfig {
    pub app_dir: PathBuf,
    pub port: u16,
}

impl ServerConfig {
    pub fn new(app_dir: impl Into<PathBuf>, port: u16) -> Self {
        Self {
            app_dir: app_dir.into(),
            port,
        }
    }
}

pub struct NextServer {
    config: ServerConfig,
    router: Router,
    registry: Arc<PageRegistry>,
}

impl NextServer {
    pub fn new(config: ServerConfig, registry: PageRegistry) -> Self {
        let scanner = RouteScanner::new(&config.app_dir);
        let routes = scanner.scan();
        let router = Router::from_routes(routes);

        Self {
            config,
            router,
            registry: Arc::new(registry),
        }
    }

    pub fn addr(&self) -> SocketAddr {
        SocketAddr::from(([127, 0, 0, 1], self.config.port))
    }

    pub fn router(&self) -> &Router {
        &self.router
    }

    pub async fn run(self) -> anyhow::Result<()> {
        let addr = self.addr();
        let listener = TcpListener::bind(addr).await?;

        let handler = Arc::new(RequestHandler::new(
            self.router,
            self.config.app_dir.clone(),
            self.registry,
        ));

        loop {
            let (stream, _) = listener.accept().await?;
            let io = TokioIo::new(stream);
            let handler = handler.clone();

            tokio::spawn(async move {
                let service = service_fn(move |req| {
                    let handler = handler.clone();
                    async move { handler.handle(req).await }
                });

                if let Err(e) = http1::Builder::new().serve_connection(io, service).await {
                    eprintln!("Connection error: {}", e);
                }
            });
        }
    }
}

pub struct DevServer {
    inner: NextServer,
    reload_tx: tokio::sync::broadcast::Sender<String>,
}

impl DevServer {
    pub fn new(config: ServerConfig, registry: PageRegistry) -> Self {
        let (reload_tx, _) = tokio::sync::broadcast::channel(16);
        Self {
            inner: NextServer::new(config, registry),
            reload_tx,
        }
    }

    pub fn addr(&self) -> SocketAddr {
        self.inner.addr()
    }

    pub fn router(&self) -> &Router {
        self.inner.router()
    }

    pub fn reload_sender(&self) -> tokio::sync::broadcast::Sender<String> {
        self.reload_tx.clone()
    }

    pub async fn run(self) -> anyhow::Result<()> {
        println!("Development server running at http://{}", self.addr());

        let addr = self.addr();
        let listener = TcpListener::bind(addr).await?;

        let handler = Arc::new(RequestHandler::new(
            self.inner.router,
            self.inner.config.app_dir.clone(),
            self.inner.registry,
        ));

        let reload_tx = self.reload_tx;

        loop {
            let (stream, _) = listener.accept().await?;
            let io = TokioIo::new(stream);
            let handler = handler.clone();
            let reload_rx = reload_tx.subscribe();

            tokio::spawn(async move {
                let service = service_fn(move |req| {
                    let handler = handler.clone();
                    let reload_rx = reload_rx.resubscribe();
                    async move { handler.handle_with_dev_ws(req, Some(reload_rx)).await }
                });

                if let Err(e) = http1::Builder::new().serve_connection(io, service).await {
                    if !e.to_string().contains("connection closed") {
                        eprintln!("Connection error: {}", e);
                    }
                }
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use tempfile::TempDir;

    fn create_test_app() -> TempDir {
        let temp = TempDir::new().unwrap();
        let app = temp.path().join("app");

        fs::create_dir_all(&app).unwrap();
        File::create(app.join("page.rs")).unwrap();
        File::create(app.join("layout.rs")).unwrap();

        fs::create_dir_all(app.join("about")).unwrap();
        File::create(app.join("about/page.rs")).unwrap();

        temp
    }

    #[test]
    fn test_server_creation() {
        let temp = create_test_app();
        let config = ServerConfig::new(temp.path().join("app"), 3000);
        let registry = PageRegistry::new();
        let server = NextServer::new(config, registry);

        assert_eq!(server.addr().port(), 3000);
        assert_eq!(server.router().routes.len(), 2);
    }

    #[test]
    fn test_dev_server_creation() {
        let temp = create_test_app();
        let config = ServerConfig::new(temp.path().join("app"), 3001);
        let registry = PageRegistry::new();
        let server = DevServer::new(config, registry);

        assert_eq!(server.addr().port(), 3001);
    }
}
