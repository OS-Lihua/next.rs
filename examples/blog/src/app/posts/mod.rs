use react_rs_elements::html::*;
use react_rs_elements::Element;

use crate::components::post_card;
use crate::data::get_all_posts;

pub fn posts_page() -> Element {
    div()
        .class("posts-page")
        .child(h1().text("Blog Posts"))
        .child(p().text("All articles about Rust, next.rs, and web development"))
        .child({
            let posts = get_all_posts();
            let mut list = div().class("post-list");
            for post in &posts {
                list = list.child(post_card(post));
            }
            list
        })
}

pub fn post_page(slug: &str) -> Element {
    use crate::components::post_detail;
    use crate::data::get_post_by_slug;

    match get_post_by_slug(slug) {
        Some(post) => post_detail(&post),
        None => div()
            .class("not-found")
            .child(h1().text("Post Not Found"))
            .child(p().text("The post you're looking for doesn't exist."))
            .child(a().attr("href", "/posts").text("Back to posts")),
    }
}
