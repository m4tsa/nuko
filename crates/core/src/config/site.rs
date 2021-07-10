use anyhow::Result;
use serde_derive::{Deserialize, Serialize};
use std::{fs, path::Path};

fn default_title_sep() -> char {
    '|'
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SiteSiteConfig {
    pub title: Option<String>,
    pub description: Option<String>,
    pub theme: Option<String>,
    pub base_url: String,
    #[serde(default = "default_title_sep")]
    pub title_seperator: char,
    pub syntax_theme: Option<String>,
    pub syntax_theme_dark: Option<String>,
    pub copy_files: Option<Vec<[String; 2]>>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SiteFeedsConfig {
    pub atom: bool,
    pub rss: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SiteConfig {
    pub site: SiteSiteConfig,
    #[serde(default)]
    pub feeds: SiteFeedsConfig,
}

impl SiteConfig {
    pub fn parse(text: &str) -> Result<SiteConfig> {
        let config: SiteConfig = toml::from_str(text)?;

        Ok(config)
    }

    pub fn read_file(path: &Path) -> Result<SiteConfig> {
        let text = fs::read_to_string(path)?;

        Self::parse(&text)
    }
}
