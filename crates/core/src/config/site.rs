use anyhow::Result;
use serde_derive::{Deserialize, Serialize};
use std::{fs, path::Path};

fn default_title_sep() -> char {
    '|'
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SiteSiteConfig {
    pub title: Option<String>,
    pub theme: Option<String>,
    #[serde(default = "default_title_sep")]
    pub title_seperator: char,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SiteConfig {
    pub site: SiteSiteConfig,
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
