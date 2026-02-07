use react_rs_elements::html::{div, main_el};
use react_rs_elements::node::IntoNode;
use react_rs_elements::Element;

use crate::components::{footer, header};

pub fn root_layout(children: impl IntoNode) -> Element {
    div()
        .class("app")
        .child(header())
        .child(main_el().class("main").child(children))
        .child(footer())
}
