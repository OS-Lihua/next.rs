use next_rs_rsc::{async_server_component, client_component, server_component, RscPayload};
use react_rs_elements::html::*;
use react_rs_elements::Element;

use crate::components::{post_card, post_detail};
use crate::data::{get_all_posts, get_post_by_slug, Post};

pub fn article_list_server() -> impl Fn() -> Element + 'static {
    || {
        let posts = get_all_posts();
        let mut list = div().class("article-list");
        for post in &posts {
            list = list.child(post_card(post));
        }
        list
    }
}

pub fn create_article_list_component(
) -> next_rs_rsc::Component<next_rs_rsc::Server, impl Fn() -> Element + 'static> {
    server_component("article-list", article_list_server())
}

pub fn article_detail_server(slug: String) -> impl Fn() -> Element + 'static {
    move || {
        if let Some(post) = get_post_by_slug(&slug) {
            post_detail(&post)
        } else {
            div().class("not-found").child(h1().text("Post not found"))
        }
    }
}

pub fn create_article_detail_component(
    slug: String,
) -> next_rs_rsc::Component<next_rs_rsc::Server, impl Fn() -> Element + 'static> {
    let id = format!("article-detail-{}", slug);
    server_component(id, article_detail_server(slug))
}

pub fn like_button_client() -> impl Fn() -> Element + 'static {
    || {
        button()
            .class("like-button")
            .attr("data-client", "true")
            .text("Like")
    }
}

pub fn create_like_button_component() -> next_rs_rsc::ClientMarker<impl Fn() -> Element + 'static> {
    client_component(
        "like-button",
        "./components/LikeButton.js",
        like_button_client(),
    )
}

pub fn render_home_page_rsc() -> RscPayload {
    let component = create_article_list_component();
    component.render_to_payload()
}

pub fn render_post_page_rsc(slug: &str) -> RscPayload {
    let component = create_article_detail_component(slug.to_string());
    component.render_to_payload()
}

async fn fetch_posts_async() -> Vec<Post> {
    get_all_posts()
}

pub fn create_async_article_list() -> next_rs_rsc::AsyncServerComponent {
    async_server_component("async-article-list", || async {
        let posts = fetch_posts_async().await;
        let mut list = div().class("async-article-list");
        for post in &posts {
            list = list.child(post_card(post));
        }
        list
    })
}

pub async fn render_async_home_page_rsc() -> RscPayload {
    let component = create_async_article_list();
    component.render_to_payload().await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_article_list_server_component() {
        let component = create_article_list_component();
        assert_eq!(component.id(), "article-list");

        let element = component.render();
        assert_eq!(element.tag(), "div");
    }

    #[test]
    fn test_article_detail_server_component() {
        let component = create_article_detail_component("getting-started-with-rust".to_string());
        assert!(component.id().starts_with("article-detail-"));

        let element = component.render();
        assert_eq!(element.tag(), "article");
    }

    #[test]
    fn test_article_detail_not_found() {
        let component = create_article_detail_component("non-existent".to_string());
        let element = component.render();
        assert!(element.has_class("not-found"));
    }

    #[test]
    fn test_like_button_client_component() {
        let component = create_like_button_component();
        assert_eq!(component.id(), "like-button");
        assert_eq!(component.module(), "./components/LikeButton.js");

        let fallback = component.render_fallback();
        assert_eq!(fallback.tag(), "button");
    }

    #[test]
    fn test_render_home_page_rsc() {
        let payload = render_home_page_rsc();
        assert!(!payload.nodes.is_empty());

        let wire = payload.to_wire_format();
        assert!(wire.contains("article-list") || wire.contains("div"));
    }

    #[test]
    fn test_render_post_page_rsc() {
        let payload = render_post_page_rsc("building-web-apps-with-next-rs");
        assert!(!payload.nodes.is_empty());
    }

    #[test]
    fn test_like_button_rsc_reference() {
        let component = create_like_button_component();
        let rsc_node = component.to_rsc_reference(serde_json::json!({"post_id": "abc"}));

        if let next_rs_rsc::RscNode::ClientReference { id, props } = rsc_node {
            assert_eq!(id, "like-button");
            assert_eq!(props["post_id"], "abc");
        } else {
            panic!("Expected ClientReference");
        }
    }

    #[tokio::test]
    async fn test_async_article_list() {
        let component = super::create_async_article_list();
        let element = component.render().await;
        assert_eq!(element.tag(), "div");
    }
}
