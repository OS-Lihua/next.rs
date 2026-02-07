use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, Response};

#[derive(Debug, Clone)]
pub struct FetchResponse {
    pub status: u16,
    pub ok: bool,
    pub body: String,
}

impl FetchResponse {
    pub fn json<T: serde::de::DeserializeOwned>(&self) -> Result<T, serde_json::Error> {
        serde_json::from_str(&self.body)
    }
}

#[derive(Debug, Clone)]
pub struct FetchError {
    pub message: String,
}

impl std::fmt::Display for FetchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl From<JsValue> for FetchError {
    fn from(value: JsValue) -> Self {
        FetchError {
            message: format!("{:?}", value),
        }
    }
}

pub async fn fetch(url: &str) -> Result<FetchResponse, FetchError> {
    fetch_with_options(url, "GET", None).await
}

pub async fn fetch_json<T: serde::de::DeserializeOwned>(url: &str) -> Result<T, FetchError> {
    let response = fetch(url).await?;
    response.json().map_err(|e| FetchError {
        message: e.to_string(),
    })
}

pub async fn post_json<T: serde::Serialize>(
    url: &str,
    body: &T,
) -> Result<FetchResponse, FetchError> {
    let json = serde_json::to_string(body).map_err(|e| FetchError {
        message: e.to_string(),
    })?;
    fetch_with_options(url, "POST", Some(&json)).await
}

async fn fetch_with_options(
    url: &str,
    method: &str,
    body: Option<&str>,
) -> Result<FetchResponse, FetchError> {
    let window = web_sys::window().ok_or_else(|| FetchError {
        message: "no window".to_string(),
    })?;

    let opts = RequestInit::new();
    opts.set_method(method);

    if let Some(body_str) = body {
        opts.set_body(&JsValue::from_str(body_str));
    }

    let request = Request::new_with_str_and_init(url, &opts)?;

    if body.is_some() {
        request.headers().set("Content-Type", "application/json")?;
    }

    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;
    let resp: Response = resp_value.dyn_into()?;

    let status = resp.status();
    let ok = resp.ok();

    let text_promise = resp.text()?;
    let text_value = JsFuture::from(text_promise).await?;
    let body = text_value.as_string().unwrap_or_default();

    Ok(FetchResponse { status, ok, body })
}

pub fn use_fetch<T, F>(url: &str, on_result: F)
where
    T: serde::de::DeserializeOwned + 'static,
    F: Fn(Result<T, FetchError>) + 'static,
{
    let url = url.to_string();
    wasm_bindgen_futures::spawn_local(async move {
        let result = fetch_json::<T>(&url).await;
        on_result(result);
    });
}
