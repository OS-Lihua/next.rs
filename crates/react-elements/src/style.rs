use std::collections::BTreeMap;

#[derive(Debug, Clone, Default)]
pub struct Style {
    properties: BTreeMap<String, String>,
}

impl Style {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set(mut self, property: &str, value: &str) -> Self {
        self.properties
            .insert(property.to_string(), value.to_string());
        self
    }

    pub fn display(self, value: &str) -> Self {
        self.set("display", value)
    }
    pub fn position(self, value: &str) -> Self {
        self.set("position", value)
    }
    pub fn width(self, value: &str) -> Self {
        self.set("width", value)
    }
    pub fn height(self, value: &str) -> Self {
        self.set("height", value)
    }
    pub fn max_width(self, value: &str) -> Self {
        self.set("max-width", value)
    }
    pub fn min_height(self, value: &str) -> Self {
        self.set("min-height", value)
    }
    pub fn margin(self, value: &str) -> Self {
        self.set("margin", value)
    }
    pub fn margin_top(self, value: &str) -> Self {
        self.set("margin-top", value)
    }
    pub fn margin_bottom(self, value: &str) -> Self {
        self.set("margin-bottom", value)
    }
    pub fn padding(self, value: &str) -> Self {
        self.set("padding", value)
    }
    pub fn background(self, value: &str) -> Self {
        self.set("background", value)
    }
    pub fn background_color(self, value: &str) -> Self {
        self.set("background-color", value)
    }
    pub fn color(self, value: &str) -> Self {
        self.set("color", value)
    }
    pub fn font_size(self, value: &str) -> Self {
        self.set("font-size", value)
    }
    pub fn font_weight(self, value: &str) -> Self {
        self.set("font-weight", value)
    }
    pub fn font_family(self, value: &str) -> Self {
        self.set("font-family", value)
    }
    pub fn line_height(self, value: &str) -> Self {
        self.set("line-height", value)
    }
    pub fn text_align(self, value: &str) -> Self {
        self.set("text-align", value)
    }
    pub fn border(self, value: &str) -> Self {
        self.set("border", value)
    }
    pub fn border_radius(self, value: &str) -> Self {
        self.set("border-radius", value)
    }
    pub fn box_shadow(self, value: &str) -> Self {
        self.set("box-shadow", value)
    }
    pub fn cursor(self, value: &str) -> Self {
        self.set("cursor", value)
    }
    pub fn overflow(self, value: &str) -> Self {
        self.set("overflow", value)
    }
    pub fn opacity(self, value: &str) -> Self {
        self.set("opacity", value)
    }
    pub fn transition(self, value: &str) -> Self {
        self.set("transition", value)
    }
    pub fn transform(self, value: &str) -> Self {
        self.set("transform", value)
    }
    pub fn gap(self, value: &str) -> Self {
        self.set("gap", value)
    }
    pub fn flex(self, value: &str) -> Self {
        self.set("flex", value)
    }
    pub fn flex_direction(self, value: &str) -> Self {
        self.set("flex-direction", value)
    }
    pub fn align_items(self, value: &str) -> Self {
        self.set("align-items", value)
    }
    pub fn justify_content(self, value: &str) -> Self {
        self.set("justify-content", value)
    }
    pub fn grid_template_columns(self, value: &str) -> Self {
        self.set("grid-template-columns", value)
    }

    pub fn to_css(&self) -> String {
        self.properties
            .iter()
            .map(|(k, v)| format!("{}: {}", k, v))
            .collect::<Vec<_>>()
            .join("; ")
    }
}

pub fn style() -> Style {
    Style::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_style_builder() {
        let s = style().display("flex").gap("10px").padding("20px");
        assert_eq!(s.to_css(), "display: flex; gap: 10px; padding: 20px");
    }

    #[test]
    fn test_style_empty() {
        let s = style();
        assert_eq!(s.to_css(), "");
    }

    #[test]
    fn test_style_single() {
        let s = style().color("#333");
        assert_eq!(s.to_css(), "color: #333");
    }

    #[test]
    fn test_style_complex() {
        let s = style()
            .display("grid")
            .grid_template_columns("1fr 1fr")
            .gap("16px")
            .padding("24px")
            .background_color("#f8f9fa")
            .border_radius("8px");
        let css = s.to_css();
        assert!(css.contains("display: grid"));
        assert!(css.contains("grid-template-columns: 1fr 1fr"));
        assert!(css.contains("border-radius: 8px"));
    }
}
