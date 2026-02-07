use react_rs_elements::html::*;
use react_rs_elements::Element;

use crate::components::post_card;
use crate::data::get_all_posts;

pub fn home_page() -> Element {
    div()
        .class("home")
        .child(
            section()
                .class("hero")
                .child(h1().text("Welcome to the Blog"))
                .child(p().text("A blog built with next.rs - Next.js reimplemented in Rust")),
        )
        .child(
            section()
                .class("recent-posts")
                .child(h2().text("Recent Posts"))
                .child({
                    let posts = get_all_posts();
                    let mut list = div().class("post-list");
                    for post in posts.iter().take(3) {
                        list = list.child(post_card(post));
                    }
                    list
                }),
        )
}
