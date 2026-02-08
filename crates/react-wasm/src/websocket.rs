use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{CloseEvent, MessageEvent, WebSocket};

pub struct WsHandle {
    ws: WebSocket,
}

impl WsHandle {
    pub fn send_text(&self, text: &str) {
        let _ = self.ws.send_with_str(text);
    }

    pub fn send_binary(&self, data: &[u8]) {
        let _ = self.ws.send_with_u8_array(data);
    }

    pub fn close(&self) {
        let _ = self.ws.close();
    }

    pub fn ready_state(&self) -> u16 {
        self.ws.ready_state()
    }

    pub fn is_open(&self) -> bool {
        self.ws.ready_state() == WebSocket::OPEN
    }
}

pub fn use_websocket<FMsg, FOpen, FClose, FErr>(
    url: &str,
    on_message: FMsg,
    on_open: Option<FOpen>,
    on_close: Option<FClose>,
    on_error: Option<FErr>,
) -> WsHandle
where
    FMsg: Fn(String) + 'static,
    FOpen: Fn() + 'static,
    FClose: Fn(u16, String) + 'static,
    FErr: Fn(String) + 'static,
{
    let ws = WebSocket::new(url).expect("Failed to create WebSocket");
    ws.set_binary_type(web_sys::BinaryType::Arraybuffer);

    let on_message_cb = Closure::<dyn FnMut(MessageEvent)>::new(move |e: MessageEvent| {
        if let Some(text) = e.data().as_string() {
            on_message(text);
        }
    });
    ws.set_onmessage(Some(on_message_cb.as_ref().unchecked_ref()));
    on_message_cb.forget();

    if let Some(on_open) = on_open {
        let on_open_cb = Closure::<dyn FnMut()>::new(move || {
            on_open();
        });
        ws.set_onopen(Some(on_open_cb.as_ref().unchecked_ref()));
        on_open_cb.forget();
    }

    if let Some(on_close) = on_close {
        let on_close_cb = Closure::<dyn FnMut(CloseEvent)>::new(move |e: CloseEvent| {
            on_close(e.code(), e.reason());
        });
        ws.set_onclose(Some(on_close_cb.as_ref().unchecked_ref()));
        on_close_cb.forget();
    }

    if let Some(on_error) = on_error {
        let on_error_cb =
            Closure::<dyn FnMut(web_sys::ErrorEvent)>::new(move |e: web_sys::ErrorEvent| {
                on_error(e.message());
            });
        ws.set_onerror(Some(on_error_cb.as_ref().unchecked_ref()));
        on_error_cb.forget();
    }

    WsHandle { ws }
}

pub fn use_websocket_simple(url: &str, on_message: impl Fn(String) + 'static) -> WsHandle {
    use_websocket::<_, fn(), fn(u16, String), fn(String)>(url, on_message, None, None, None)
}
