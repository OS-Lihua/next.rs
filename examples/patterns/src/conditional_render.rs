use react_rs_core::create_signal;
use react_rs_elements::html::*;
use react_rs_elements::node::IntoNode;
use react_rs_elements::SignalExt;

pub fn login_gate() -> impl IntoNode {
    let (logged_in, set_logged_in) = create_signal(false);

    div()
        .child(h1().text("Login Example"))
        .child(p().text_reactive(logged_in.map(|v| {
            if *v {
                "Welcome back!".to_string()
            } else {
                "Please log in.".to_string()
            }
        })))
        .child(button().text("Log In").on_click(move |_| {
            set_logged_in.set(true);
        }))
}
