use react_rs_core::create_signal;
use react_rs_elements::html::*;
use react_rs_elements::node::IntoNode;
use react_rs_elements::SignalExt;

pub fn validated_form() -> impl IntoNode {
    let (email, set_email) = create_signal(String::new());
    let (status, set_status) = create_signal("Enter your email".to_string());

    let status_display = status.map(|s| s.clone());
    let email_for_submit = email.clone();

    form()
        .on_submit(move |_e| {
            let val = email_for_submit.get();
            if val.contains('@') && val.contains('.') {
                set_status.set(format!("Submitted: {}", val));
            } else {
                set_status.set("Invalid email address".to_string());
            }
        })
        .child(
            input()
                .type_("email")
                .placeholder("you@example.com")
                .bind_value(email, set_email),
        )
        .child(p().text_reactive(status_display))
        .child(button().type_("submit").text("Submit"))
}
