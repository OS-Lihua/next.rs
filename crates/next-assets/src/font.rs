use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontConfig {
    pub preload: bool,
    pub display: FontDisplay,
    pub subsets: Vec<String>,
    pub variable: Option<String>,
}

impl Default for FontConfig {
    fn default() -> Self {
        Self {
            preload: true,
            display: FontDisplay::Swap,
            subsets: vec!["latin".to_string()],
            variable: None,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum FontDisplay {
    Auto,
    Block,
    Swap,
    Fallback,
    Optional,
}

impl FontDisplay {
    pub fn as_str(&self) -> &'static str {
        match self {
            FontDisplay::Auto => "auto",
            FontDisplay::Block => "block",
            FontDisplay::Swap => "swap",
            FontDisplay::Fallback => "fallback",
            FontDisplay::Optional => "optional",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum FontWeight {
    Thin,
    ExtraLight,
    Light,
    Regular,
    Medium,
    SemiBold,
    Bold,
    ExtraBold,
    Black,
    Variable(u16, u16),
}

impl FontWeight {
    pub fn value(&self) -> String {
        match self {
            FontWeight::Thin => "100".to_string(),
            FontWeight::ExtraLight => "200".to_string(),
            FontWeight::Light => "300".to_string(),
            FontWeight::Regular => "400".to_string(),
            FontWeight::Medium => "500".to_string(),
            FontWeight::SemiBold => "600".to_string(),
            FontWeight::Bold => "700".to_string(),
            FontWeight::ExtraBold => "800".to_string(),
            FontWeight::Black => "900".to_string(),
            FontWeight::Variable(min, max) => format!("{} {}", min, max),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Font {
    pub family: String,
    pub src: FontSource,
    pub weight: Vec<FontWeight>,
    pub style: FontStyle,
    pub display: FontDisplay,
    pub preload: bool,
    pub variable: Option<String>,
    pub fallback: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum FontSource {
    Google(String),
    Local(Vec<String>),
    Url(String),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FontStyle {
    Normal,
    Italic,
}

impl Font {
    pub fn google(family: impl Into<String>) -> Self {
        let family_str = family.into();
        Self {
            family: family_str.clone(),
            src: FontSource::Google(family_str),
            weight: vec![FontWeight::Regular],
            style: FontStyle::Normal,
            display: FontDisplay::Swap,
            preload: true,
            variable: None,
            fallback: vec!["sans-serif".to_string()],
        }
    }

    pub fn local(family: impl Into<String>, files: Vec<String>) -> Self {
        Self {
            family: family.into(),
            src: FontSource::Local(files),
            weight: vec![FontWeight::Regular],
            style: FontStyle::Normal,
            display: FontDisplay::Swap,
            preload: true,
            variable: None,
            fallback: vec!["sans-serif".to_string()],
        }
    }

    pub fn with_weights(mut self, weights: Vec<FontWeight>) -> Self {
        self.weight = weights;
        self
    }

    pub fn with_style(mut self, style: FontStyle) -> Self {
        self.style = style;
        self
    }

    pub fn with_display(mut self, display: FontDisplay) -> Self {
        self.display = display;
        self
    }

    pub fn with_variable(mut self, var_name: impl Into<String>) -> Self {
        self.variable = Some(var_name.into());
        self
    }

    pub fn with_fallback(mut self, fallback: Vec<String>) -> Self {
        self.fallback = fallback;
        self
    }

    pub fn css_family(&self) -> String {
        if let Some(var) = &self.variable {
            format!("var({})", var)
        } else {
            let fallback = self.fallback.join(", ");
            format!("'{}', {}", self.family, fallback)
        }
    }

    pub fn google_fonts_url(&self) -> Option<String> {
        match &self.src {
            FontSource::Google(family) => {
                let weights: Vec<String> = self.weight.iter().map(|w| w.value()).collect();
                let weight_param = if weights.len() == 1 {
                    format!("wght@{}", weights[0])
                } else {
                    format!("wght@{}", weights.join(";"))
                };

                Some(format!(
                    "https://fonts.googleapis.com/css2?family={}:{}&display={}",
                    family.replace(' ', "+"),
                    weight_param,
                    self.display.as_str()
                ))
            }
            _ => None,
        }
    }

    pub fn preload_links(&self) -> Vec<PreloadLink> {
        let mut links = Vec::new();

        if !self.preload {
            return links;
        }

        if let Some(url) = self.google_fonts_url() {
            links.push(PreloadLink {
                href: url,
                as_type: "style".to_string(),
                crossorigin: Some("anonymous".to_string()),
            });
        }

        links
    }
}

#[derive(Debug, Clone)]
pub struct PreloadLink {
    pub href: String,
    pub as_type: String,
    pub crossorigin: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_google_font() {
        let font = Font::google("Inter").with_weights(vec![FontWeight::Regular, FontWeight::Bold]);

        assert_eq!(font.family, "Inter");
        assert_eq!(font.weight.len(), 2);

        let url = font.google_fonts_url().unwrap();
        assert!(url.contains("Inter"));
        assert!(url.contains("400"));
        assert!(url.contains("700"));
    }

    #[test]
    fn test_local_font() {
        let font = Font::local("CustomFont", vec!["custom.woff2".to_string()])
            .with_variable("--font-custom");

        assert_eq!(font.family, "CustomFont");
        assert!(font.variable.is_some());
        assert!(font.css_family().contains("var(--font-custom)"));
    }

    #[test]
    fn test_font_display() {
        assert_eq!(FontDisplay::Swap.as_str(), "swap");
        assert_eq!(FontDisplay::Optional.as_str(), "optional");
    }

    #[test]
    fn test_font_weight_value() {
        assert_eq!(FontWeight::Regular.value(), "400");
        assert_eq!(FontWeight::Bold.value(), "700");
        assert_eq!(FontWeight::Variable(100, 900).value(), "100 900");
    }

    #[test]
    fn test_css_family() {
        let font = Font::google("Roboto").with_fallback(vec![
            "Helvetica".to_string(),
            "Arial".to_string(),
            "sans-serif".to_string(),
        ]);

        let family = font.css_family();
        assert!(family.contains("'Roboto'"));
        assert!(family.contains("Helvetica"));
        assert!(family.contains("sans-serif"));
    }

    #[test]
    fn test_preload_links() {
        let font = Font::google("Open Sans");
        let links = font.preload_links();

        assert!(!links.is_empty());
        assert!(links[0].href.contains("fonts.googleapis.com"));
    }
}
