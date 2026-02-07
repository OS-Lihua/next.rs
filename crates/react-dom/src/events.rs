use react_rs_elements::events::Event as ReactEvent;
use react_rs_elements::Element;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

thread_local! {
    static EVENT_CLOSURES: RefCell<Vec<Closure<dyn FnMut(web_sys::Event)>>> =
        RefCell::new(Vec::new());
}

pub fn attach_event_handlers(el: &web_sys::Element, element: &Element) {
    let event_target: &web_sys::EventTarget = el.dyn_ref().unwrap();

    for handler in element.event_handlers() {
        let event_type = handler.event_type.clone();
        let callback = handler.take_handler_rc();

        let closure = Closure::wrap(Box::new(move |e: web_sys::Event| {
            let react_event = ReactEvent::new(e.type_());
            callback(react_event);
        }) as Box<dyn FnMut(web_sys::Event)>);

        event_target
            .add_event_listener_with_callback(&event_type, closure.as_ref().unchecked_ref())
            .expect("failed to add event listener");

        EVENT_CLOSURES.with(|closures| {
            closures.borrow_mut().push(closure);
        });
    }
}
