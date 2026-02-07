pub struct Post {
    pub slug: String,
    pub title: String,
    pub excerpt: String,
    pub content: String,
    pub author: String,
    pub date: String,
}

impl Post {
    pub fn new(
        slug: impl Into<String>,
        title: impl Into<String>,
        excerpt: impl Into<String>,
        content: impl Into<String>,
        author: impl Into<String>,
        date: impl Into<String>,
    ) -> Self {
        Self {
            slug: slug.into(),
            title: title.into(),
            excerpt: excerpt.into(),
            content: content.into(),
            author: author.into(),
            date: date.into(),
        }
    }
}

pub fn get_all_posts() -> Vec<Post> {
    vec![
        Post::new(
            "getting-started-with-rust",
            "Getting Started with Rust",
            "Learn the basics of Rust programming language",
            "Rust is a systems programming language that runs blazingly fast...",
            "Alice",
            "2024-01-15",
        ),
        Post::new(
            "building-web-apps-with-next-rs",
            "Building Web Apps with next.rs",
            "A guide to building modern web applications using next.rs",
            "next.rs brings the power of Next.js to the Rust ecosystem...",
            "Bob",
            "2024-01-20",
        ),
        Post::new(
            "react-rs-deep-dive",
            "React.rs Deep Dive",
            "Understanding the internals of react.rs",
            "In this post we'll explore how react.rs implements reactivity...",
            "Charlie",
            "2024-01-25",
        ),
    ]
}

pub fn get_post_by_slug(slug: &str) -> Option<Post> {
    get_all_posts().into_iter().find(|p| p.slug == slug)
}
