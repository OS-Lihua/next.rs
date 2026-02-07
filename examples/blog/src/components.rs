use react_rs_elements::html::{
    a, article, div, footer as html_footer, h1, h2, header as html_header, nav, p, span,
};
use react_rs_elements::Element;

use crate::data::Post;

pub fn header() -> Element {
    html_header().class("site-header").child(
        nav()
            .class("nav")
            .child(a().attr("href", "/").text("Home"))
            .child(a().attr("href", "/posts").text("Posts"))
            .child(a().attr("href", "/about").text("About")),
    )
}

pub fn footer() -> Element {
    html_footer()
        .class("site-footer")
        .child(p().text("Built with next.rs - Next.js reimplemented in Rust"))
}

pub fn post_card(post: &Post) -> Element {
    let href = format!("/posts/{}", post.slug);
    let author = format!("By {}", post.author);
    article()
        .class("post-card")
        .child(h2().child(a().attr("href", &href).text(&post.title)))
        .child(p().class("excerpt").text(&post.excerpt))
        .child(
            div()
                .class("meta")
                .child(span().class("author").text(&author))
                .child(span().class("date").text(&post.date)),
        )
}

pub fn post_detail(post: &Post) -> Element {
    let author = format!("By {}", post.author);
    article()
        .class("post-detail")
        .child(h1().text(&post.title))
        .child(
            div()
                .class("meta")
                .child(span().class("author").text(&author))
                .child(span().class("date").text(&post.date)),
        )
        .child(div().class("content").child(p().text(&post.content)))
        .child(
            a().attr("href", "/posts")
                .class("back-link")
                .text("Back to all posts"),
        )
}
