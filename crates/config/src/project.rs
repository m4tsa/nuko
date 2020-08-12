use anyhow::Result;
use serde_derive::{Deserialize, Serialize};
use std::{fs, path::Path};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjectSiteConfig {
    pub title: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub site: ProjectSiteConfig,
}

impl ProjectConfig {
    pub fn parse(text: &str) -> Result<ProjectConfig> {
        let config: ProjectConfig = toml::from_str(text)?;

        Ok(config)
    }

    pub fn read_file(path: &Path) -> Result<ProjectConfig> {
        let text = fs::read_to_string(path)?;

        Self::parse(&text)
    }
}
