use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageConfig {
    pub domains: Vec<String>,
    pub device_sizes: Vec<u32>,
    pub image_sizes: Vec<u32>,
    pub formats: Vec<ImageFormat>,
    pub quality: u8,
    pub loader: ImageLoader,
}

impl Default for ImageConfig {
    fn default() -> Self {
        Self {
            domains: Vec::new(),
            device_sizes: vec![640, 750, 828, 1080, 1200, 1920, 2048, 3840],
            image_sizes: vec![16, 32, 48, 64, 96, 128, 256, 384],
            formats: vec![ImageFormat::Webp, ImageFormat::Avif],
            quality: 75,
            loader: ImageLoader::Default,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ImageFormat {
    Webp,
    Avif,
    Png,
    Jpeg,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ImageLoader {
    Default,
    Cloudinary,
    Imgix,
    Custom(String),
}

#[derive(Debug, Clone)]
pub struct Image {
    pub src: String,
    pub alt: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub priority: bool,
    pub placeholder: Placeholder,
    pub quality: Option<u8>,
    pub fill: bool,
    pub sizes: Option<String>,
}

#[derive(Debug, Clone)]
pub enum Placeholder {
    Empty,
    Blur(String),
}

impl Image {
    pub fn new(src: impl Into<String>, alt: impl Into<String>) -> Self {
        Self {
            src: src.into(),
            alt: alt.into(),
            width: None,
            height: None,
            priority: false,
            placeholder: Placeholder::Empty,
            quality: None,
            fill: false,
            sizes: None,
        }
    }

    pub fn with_size(mut self, width: u32, height: u32) -> Self {
        self.width = Some(width);
        self.height = Some(height);
        self
    }

    pub fn priority(mut self) -> Self {
        self.priority = true;
        self
    }

    pub fn with_blur_placeholder(mut self, blur_data_url: impl Into<String>) -> Self {
        self.placeholder = Placeholder::Blur(blur_data_url.into());
        self
    }

    pub fn fill(mut self) -> Self {
        self.fill = true;
        self
    }

    pub fn with_quality(mut self, quality: u8) -> Self {
        self.quality = Some(quality.min(100));
        self
    }

    pub fn with_sizes(mut self, sizes: impl Into<String>) -> Self {
        self.sizes = Some(sizes.into());
        self
    }

    pub fn optimized_url(&self, config: &ImageConfig, target_width: u32) -> String {
        let quality = self.quality.unwrap_or(config.quality);

        match &config.loader {
            ImageLoader::Default => {
                format!(
                    "/_next/image?url={}&w={}&q={}",
                    urlencoding(&self.src),
                    target_width,
                    quality
                )
            }
            ImageLoader::Cloudinary => {
                format!(
                    "https://res.cloudinary.com/demo/image/fetch/w_{},q_{}/{}",
                    target_width, quality, self.src
                )
            }
            ImageLoader::Imgix => {
                format!("{}?w={}&q={}", self.src, target_width, quality)
            }
            ImageLoader::Custom(pattern) => pattern
                .replace("{src}", &self.src)
                .replace("{width}", &target_width.to_string())
                .replace("{quality}", &quality.to_string()),
        }
    }

    pub fn srcset(&self, config: &ImageConfig) -> String {
        let widths = if self.fill {
            &config.device_sizes
        } else {
            &config.image_sizes
        };

        widths
            .iter()
            .map(|w| format!("{} {}w", self.optimized_url(config, *w), w))
            .collect::<Vec<_>>()
            .join(", ")
    }

    pub fn render_attrs(&self, config: &ImageConfig) -> Vec<(String, String)> {
        let mut attrs = vec![
            ("alt".to_string(), self.alt.clone()),
            ("srcset".to_string(), self.srcset(config)),
        ];

        if let Some(w) = self.width {
            attrs.push(("width".to_string(), w.to_string()));
        }
        if let Some(h) = self.height {
            attrs.push(("height".to_string(), h.to_string()));
        }

        if self.priority {
            attrs.push(("loading".to_string(), "eager".to_string()));
            attrs.push(("fetchpriority".to_string(), "high".to_string()));
        } else {
            attrs.push(("loading".to_string(), "lazy".to_string()));
        }

        if let Some(sizes) = &self.sizes {
            attrs.push(("sizes".to_string(), sizes.clone()));
        } else if self.fill {
            attrs.push(("sizes".to_string(), "100vw".to_string()));
        }

        if self.fill {
            attrs.push((
                "style".to_string(),
                "object-fit: cover; width: 100%; height: 100%;".to_string(),
            ));
        }

        attrs
    }
}

fn urlencoding(s: &str) -> String {
    s.replace(':', "%3A").replace('/', "%2F")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_image_creation() {
        let img = Image::new("/photo.jpg", "A photo").with_size(800, 600);

        assert_eq!(img.src, "/photo.jpg");
        assert_eq!(img.alt, "A photo");
        assert_eq!(img.width, Some(800));
        assert_eq!(img.height, Some(600));
    }

    #[test]
    fn test_image_optimized_url() {
        let img = Image::new("/photo.jpg", "Photo");
        let config = ImageConfig::default();

        let url = img.optimized_url(&config, 640);
        assert!(url.contains("/_next/image"));
        assert!(url.contains("w=640"));
        assert!(url.contains("q=75"));
    }

    #[test]
    fn test_image_srcset() {
        let img = Image::new("/photo.jpg", "Photo");
        let config = ImageConfig::default();

        let srcset = img.srcset(&config);
        assert!(srcset.contains("16w"));
        assert!(srcset.contains("256w"));
    }

    #[test]
    fn test_image_priority() {
        let img = Image::new("/hero.jpg", "Hero").priority();
        let config = ImageConfig::default();

        let attrs = img.render_attrs(&config);
        let loading = attrs.iter().find(|(k, _)| k == "loading");
        assert_eq!(loading.map(|(_, v)| v.as_str()), Some("eager"));
    }

    #[test]
    fn test_custom_loader() {
        let img = Image::new("/photo.jpg", "Photo");
        let config = ImageConfig {
            loader: ImageLoader::Custom("https://cdn.example.com/{src}?w={width}".to_string()),
            ..Default::default()
        };

        let url = img.optimized_url(&config, 800);
        assert!(url.contains("cdn.example.com"));
        assert!(url.contains("w=800"));
    }
}
