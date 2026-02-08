use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct NextConfig {
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_output_dir")]
    pub output_dir: String,
    #[serde(default = "default_tailwind")]
    pub tailwind: bool,
    #[serde(default)]
    pub images: ImageConfig,
}

impl Default for NextConfig {
    fn default() -> Self {
        Self {
            port: default_port(),
            output_dir: default_output_dir(),
            tailwind: default_tailwind(),
            images: ImageConfig::default(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ImageConfig {
    #[serde(default)]
    pub domains: Vec<String>,
    #[serde(default = "default_loader")]
    pub loader: String,
}

impl Default for ImageConfig {
    fn default() -> Self {
        Self {
            domains: Vec::new(),
            loader: default_loader(),
        }
    }
}

fn default_port() -> u16 {
    3000
}
fn default_output_dir() -> String {
    ".next".to_string()
}
fn default_tailwind() -> bool {
    true
}
fn default_loader() -> String {
    "default".to_string()
}

impl NextConfig {
    pub fn load() -> Self {
        let config_path = Path::new("next.config.toml");
        if config_path.exists() {
            match fs::read_to_string(config_path) {
                Ok(content) => match toml::from_str(&content) {
                    Ok(config) => return config,
                    Err(e) => eprintln!("Warning: Failed to parse next.config.toml: {}", e),
                },
                Err(e) => eprintln!("Warning: Failed to read next.config.toml: {}", e),
            }
        }
        Self::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = NextConfig::default();
        assert_eq!(config.port, 3000);
        assert_eq!(config.output_dir, ".next");
        assert!(config.tailwind);
    }

    #[test]
    fn test_parse_config() {
        let toml_str = r#"
port = 8080
output_dir = "dist"
tailwind = false

[images]
domains = ["cdn.example.com"]
loader = "cloudinary"
"#;
        let config: NextConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.port, 8080);
        assert_eq!(config.output_dir, "dist");
        assert!(!config.tailwind);
        assert_eq!(config.images.domains, vec!["cdn.example.com"]);
        assert_eq!(config.images.loader, "cloudinary");
    }
}
