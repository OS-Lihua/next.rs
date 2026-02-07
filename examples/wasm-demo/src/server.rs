use http_body_util::Full;
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Method, Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use react_rs_dom::render_to_string;
use std::net::SocketAddr;
use tokio::net::TcpListener;

fn render_html(route: &str) -> String {
    let node = wasm_demo::render_app(route);
    let output = render_to_string(&node);

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>next.rs WASM Demo</title>
    <style>
        * {{
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }}
        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, sans-serif;
            line-height: 1.6;
            color: #333;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            min-height: 100vh;
        }}
        .app {{
            max-width: 1200px;
            margin: 0 auto;
            padding: 20px;
        }}
        .header {{
            display: flex;
            justify-content: space-between;
            align-items: center;
            padding: 20px;
            background: rgba(255, 255, 255, 0.95);
            border-radius: 12px;
            margin-bottom: 20px;
            box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
        }}
        .logo {{
            font-size: 1.8rem;
            font-weight: bold;
            color: #667eea;
        }}
        .nav-list {{
            display: flex;
            list-style: none;
            gap: 20px;
        }}
        .nav-list a {{
            text-decoration: none;
            color: #333;
            font-weight: 500;
            padding: 8px 16px;
            border-radius: 6px;
            transition: all 0.2s;
        }}
        .nav-list a:hover {{
            background: #667eea;
            color: white;
        }}
        .main {{
            background: rgba(255, 255, 255, 0.95);
            border-radius: 12px;
            padding: 40px;
            margin-bottom: 20px;
            box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
        }}
        .hero {{
            text-align: center;
            padding: 40px 0;
        }}
        .hero-title {{
            font-size: 2.5rem;
            margin-bottom: 10px;
            color: #333;
        }}
        .hero-subtitle {{
            font-size: 1.2rem;
            color: #666;
        }}
        .features {{
            padding: 40px 0;
        }}
        .features h2 {{
            text-align: center;
            margin-bottom: 30px;
        }}
        .feature-grid {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
            gap: 20px;
        }}
        .feature-card {{
            background: #f8f9fa;
            padding: 24px;
            border-radius: 8px;
            border-left: 4px solid #667eea;
        }}
        .feature-title {{
            color: #667eea;
            margin-bottom: 8px;
        }}
        .demo-section {{
            padding: 40px 0;
            text-align: center;
        }}
        .counter-widget {{
            display: inline-block;
            padding: 30px 50px;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            border-radius: 12px;
            color: white;
            margin-top: 20px;
        }}
        .counter-display {{
            font-size: 2rem;
            margin-bottom: 20px;
        }}
        .count-value {{
            font-weight: bold;
            font-size: 2.5rem;
        }}
        .counter-buttons {{
            display: flex;
            gap: 10px;
            justify-content: center;
        }}
        .btn {{
            padding: 12px 24px;
            font-size: 1rem;
            border: none;
            border-radius: 6px;
            cursor: pointer;
            font-weight: bold;
            transition: all 0.2s;
        }}
        .btn-decrement {{
            background: #e74c3c;
            color: white;
        }}
        .btn-increment {{
            background: #27ae60;
            color: white;
        }}
        .btn-reset {{
            background: #f39c12;
            color: white;
        }}
        .btn:hover {{
            transform: scale(1.05);
            box-shadow: 0 4px 12px rgba(0, 0, 0, 0.2);
        }}
        .page {{
            padding: 20px 0;
        }}
        .page h2 {{
            margin-bottom: 20px;
            color: #333;
        }}
        .explanation {{
            margin-top: 40px;
            padding: 20px;
            background: #f8f9fa;
            border-radius: 8px;
        }}
        .about-content ul {{
            margin: 20px 0;
            padding-left: 30px;
        }}
        .about-content li {{
            margin-bottom: 8px;
        }}
        .todo-widget {{
            margin-top: 40px;
            padding: 20px;
            background: #f8f9fa;
            border-radius: 8px;
        }}
        .todo-list {{
            list-style: none;
            padding: 0;
        }}
        .todo-item {{
            padding: 10px;
            background: white;
            margin-bottom: 8px;
            border-radius: 4px;
            border-left: 3px solid #667eea;
        }}
        .todo-input-row {{
            display: flex;
            gap: 10px;
            margin-bottom: 15px;
        }}
        .todo-input {{
            flex: 1;
        }}
        .high-count-msg {{
            margin-top: 15px;
            padding: 10px 20px;
            background: #ff6b6b;
            color: white;
            border-radius: 6px;
            font-weight: bold;
            text-align: center;
        }}
        .btn-add {{
            margin-top: 15px;
            background: #667eea;
            color: white;
        }}
        .greeting-form {{
            margin-top: 30px;
            padding: 30px;
            background: rgba(255, 255, 255, 0.15);
            border-radius: 12px;
            display: inline-block;
            text-align: left;
        }}
        .form-group {{
            margin-bottom: 15px;
        }}
        .form-group label {{
            display: block;
            margin-bottom: 5px;
            font-weight: bold;
        }}
        .form-input {{
            padding: 10px 16px;
            font-size: 1rem;
            border: 2px solid rgba(255, 255, 255, 0.3);
            border-radius: 6px;
            background: rgba(255, 255, 255, 0.9);
            width: 300px;
            outline: none;
        }}
        .form-input:focus {{
            border-color: #667eea;
        }}
        .greeting-output {{
            font-size: 1.3rem;
            margin-top: 15px;
        }}
        .greeting-name {{
            font-weight: bold;
            color: #ffd700;
        }}
        .footer {{
            text-align: center;
            padding: 20px;
            color: white;
        }}
        .footer a {{
            color: white;
        }}
        .not-found-page {{
            text-align: center;
            padding: 60px 0;
        }}
        .not-found-page a {{
            display: inline-block;
            margin-top: 20px;
            padding: 12px 24px;
            background: #667eea;
            color: white;
            text-decoration: none;
            border-radius: 6px;
        }}
    </style>
</head>
<body>
    <div id="app">{}</div>
    <script type="module">
        import init from './pkg/wasm_demo.js';
        init().then(() => {{
            console.log('WASM loaded and hydration started');
        }}).catch(err => {{
            console.error('Failed to load WASM:', err);
        }});
    </script>
</body>
</html>"#,
        output.html
    )
}

async fn handle_request(
    req: Request<hyper::body::Incoming>,
) -> Result<Response<Full<Bytes>>, hyper::Error> {
    let path = req.uri().path();

    match (req.method(), path) {
        (&Method::GET, path) if path.starts_with("/pkg/") => {
            let file_path = format!("./pkg{}", &path[4..]);
            match std::fs::read(&file_path) {
                Ok(content) => {
                    let content_type = if path.ends_with(".js") {
                        "application/javascript"
                    } else if path.ends_with(".wasm") {
                        "application/wasm"
                    } else {
                        "application/octet-stream"
                    };

                    Ok(Response::builder()
                        .status(StatusCode::OK)
                        .header("Content-Type", content_type)
                        .header("Access-Control-Allow-Origin", "*")
                        .body(Full::new(Bytes::from(content)))
                        .unwrap())
                }
                Err(_) => Ok(Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(Full::new(Bytes::from("File not found")))
                    .unwrap()),
            }
        }

        (&Method::GET, path) => {
            let html = render_html(path);
            Ok(Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "text/html; charset=utf-8")
                .body(Full::new(Bytes::from(html)))
                .unwrap())
        }

        _ => Ok(Response::builder()
            .status(StatusCode::METHOD_NOT_ALLOWED)
            .body(Full::new(Bytes::from("Method not allowed")))
            .unwrap()),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = TcpListener::bind(addr).await?;

    println!("===========================================");
    println!("  next.rs WASM Demo Server");
    println!("===========================================");
    println!();
    println!("  Server running at: http://{}", addr);
    println!();
    println!("  Available routes:");
    println!("    /         - Home page with features");
    println!("    /counter  - Interactive counter demo");
    println!("    /about    - About next.rs");
    println!();
    println!("  Press Ctrl+C to stop");
    println!("===========================================");

    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);

        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(io, service_fn(handle_request))
                .await
            {
                eprintln!("Error serving connection: {:?}", err);
            }
        });
    }
}
