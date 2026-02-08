use react_rs_core::{create_signal, provide_context, use_context};
use react_rs_elements::html::*;
use react_rs_elements::node::IntoNode;
use react_rs_elements::SignalExt;

#[derive(Clone)]
pub struct Theme {
    pub dark: bool,
}

pub fn app_with_theme() -> impl IntoNode {
    let (dark, set_dark) = create_signal(false);

    provide_context(Theme { dark: false });

    div()
        .class_reactive(dark.map(|d| {
            if *d {
                "bg-gray-900 text-white min-h-screen".to_string()
            } else {
                "bg-white text-black min-h-screen".to_string()
            }
        }))
        .child(
            button()
                .text_reactive(dark.map(|d| {
                    if *d {
                        "Switch to Light".to_string()
                    } else {
                        "Switch to Dark".to_string()
                    }
                }))
                .on_click(move |_| {
                    set_dark.update(|d| *d = !*d);
                }),
        )
        .child(themed_card())
}

fn themed_card() -> impl IntoNode {
    let _theme: Option<Theme> = use_context();
    div()
        .class("p-4 border rounded m-4")
        .child(h2().text("Themed Card"))
        .child(p().text("This card respects the theme context."))
}
