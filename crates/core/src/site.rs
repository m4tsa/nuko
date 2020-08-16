use crate::config::SiteConfig;
use anyhow::Result;
use std::path::{Path, PathBuf};

pub struct Site {
    site_config: SiteConfig,
    root_path: PathBuf,
}

impl Site {
    pub fn new(root_path: &Path, site_config: SiteConfig) -> Result<Site> {
        Ok(Site {
            root_path: root_path.into(),
            site_config,
        })
    }
}
