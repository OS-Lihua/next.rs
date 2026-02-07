use react_rs_core::signal::{create_signal, ReadSignal, WriteSignal};
use std::cell::RefCell;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

thread_local! {
    static ROUTER: RefCell<Option<RouterInner>> = const { RefCell::new(None) };
}

struct RouterInner {
    current_path: ReadSignal<String>,
    set_path: WriteSignal<String>,
    #[allow(dead_code)]
    popstate_closure: Closure<dyn FnMut(web_sys::Event)>,
}

pub struct Router;

impl Router {
    pub fn init() {
        ROUTER.with(|r| {
            if r.borrow().is_some() {
                return;
            }

            let initial_path = get_current_path();
            let (current_path, set_path) = create_signal(initial_path);

            let set_path_clone = set_path.clone();
            let popstate_closure = Closure::wrap(Box::new(move |_: web_sys::Event| {
                let path = get_current_path();
                set_path_clone.set(path);
            }) as Box<dyn FnMut(web_sys::Event)>);

            web_sys::window()
                .expect("no window")
                .add_event_listener_with_callback(
                    "popstate",
                    popstate_closure.as_ref().unchecked_ref(),
                )
                .expect("failed to add popstate listener");

            *r.borrow_mut() = Some(RouterInner {
                current_path,
                set_path,
                popstate_closure,
            });
        });
    }
}

fn get_current_path() -> String {
    web_sys::window()
        .expect("no window")
        .location()
        .pathname()
        .unwrap_or_else(|_| "/".to_string())
}

pub fn use_location() -> ReadSignal<String> {
    ROUTER.with(|r| {
        let router_ref = r.borrow();
        router_ref
            .as_ref()
            .map(|inner| inner.current_path.clone())
            .expect("Router not initialized. Call Router::init() first.")
    })
}

pub fn navigate(path: &str) {
    ROUTER.with(|r| {
        let router_ref = r.borrow();
        if let Some(inner) = router_ref.as_ref() {
            let window = web_sys::window().expect("no window");
            let history = window.history().expect("no history");

            history
                .push_state_with_url(&JsValue::NULL, "", Some(path))
                .expect("failed to push state");

            inner.set_path.set(path.to_string());
        }
    });
}

pub fn replace(path: &str) {
    ROUTER.with(|r| {
        let router_ref = r.borrow();
        if let Some(inner) = router_ref.as_ref() {
            let window = web_sys::window().expect("no window");
            let history = window.history().expect("no history");

            history
                .replace_state_with_url(&JsValue::NULL, "", Some(path))
                .expect("failed to replace state");

            inner.set_path.set(path.to_string());
        }
    });
}

pub fn back() {
    if let Some(window) = web_sys::window() {
        if let Ok(history) = window.history() {
            let _ = history.back();
        }
    }
}

pub fn forward() {
    if let Some(window) = web_sys::window() {
        if let Ok(history) = window.history() {
            let _ = history.forward();
        }
    }
}

pub fn setup_link_interception() {
    let document = web_sys::window()
        .expect("no window")
        .document()
        .expect("no document");

    let closure = Closure::wrap(Box::new(move |e: web_sys::Event| {
        let target = match e.target() {
            Some(t) => t,
            None => return,
        };

        let mut current: Option<web_sys::Element> = target.dyn_ref::<web_sys::Element>().cloned();

        while let Some(el) = current {
            if el.tag_name().to_lowercase() == "a" {
                if let Some(anchor) = el.dyn_ref::<web_sys::HtmlAnchorElement>() {
                    if let Some(href) = anchor.get_attribute("href") {
                        if href.starts_with('/') && !href.starts_with("//") {
                            e.prevent_default();
                            navigate(&href);
                            return;
                        }
                    }
                }
            }
            current = el.parent_element();
        }
    }) as Box<dyn FnMut(web_sys::Event)>);

    document
        .add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())
        .expect("failed to add click listener");

    closure.forget();
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_router_module_compiles() {
        let _ = 1 + 1;
    }
}
