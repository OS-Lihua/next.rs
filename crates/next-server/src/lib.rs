mod api;
mod handler;
mod isr;
mod rsc_handler;
mod ssg;
mod ssr;
mod streaming;

pub use api::{ApiRequest, ApiResponse, ApiRouteHandler};
pub use handler::RequestHandler;
pub use isr::{CacheEntry, IncrementalCache, IsrConfig};
pub use rsc_handler::RscHandler;
pub use ssg::{GeneratedFile, GenerationResult, StaticGenerator, StaticParams};
pub use ssr::SsrRenderer;
pub use streaming::{HtmlStream, RscStream, RscStreamingRenderer, StreamingRenderer};

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
}

impl NextServer {
    pub fn new(config: ServerConfig) -> Self {
        let scanner = RouteScanner::new(&config.app_dir);
        let routes = scanner.scan();
        let router = Router::from_routes(routes);

        Self { config, router }
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
}

impl DevServer {
    pub fn new(config: ServerConfig) -> Self {
        Self {
            inner: NextServer::new(config),
        }
    }

    pub fn addr(&self) -> SocketAddr {
        self.inner.addr()
    }

    pub fn router(&self) -> &Router {
        self.inner.router()
    }

    pub async fn run(self) -> anyhow::Result<()> {
        println!("Development server running at http://{}", self.addr());
        self.inner.run().await
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
        let server = NextServer::new(config);

        assert_eq!(server.addr().port(), 3000);
        assert_eq!(server.router().routes.len(), 2);
    }

    #[test]
    fn test_dev_server_creation() {
        let temp = create_test_app();
        let config = ServerConfig::new(temp.path().join("app"), 3001);
        let server = DevServer::new(config);

        assert_eq!(server.addr().port(), 3001);
    }
}
