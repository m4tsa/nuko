use crate::{config::SiteConfig, page::Page};
use anyhow::Result;
use glob::glob;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};
use thiserror::Error;

pub struct Site {
    site_config: SiteConfig,
    root_path: PathBuf,
    pages: HashMap<String, Page>,
}

impl Site {
    pub fn new(root_path: &Path, site_config: SiteConfig) -> Result<Site> {
        if !root_path.is_absolute() {
            return Err(SiteError::NonAbsoluteRoot.into());
        }

        Ok(Site {
            root_path: root_path.into(),
            site_config,
            pages: HashMap::new(),
        })
    }

    pub fn load_content(&mut self) -> Result<()> {
        let content_dir = self.root_path.join("content");

        let content: Vec<PathBuf> = glob(&format!("{}/**/*.org", content_dir.to_string_lossy()))?
            .filter_map(|p| p.ok())
            .filter(|e| {
                !e.as_path()
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .starts_with('.')
            })
            .collect();

        println!("{:?}", content);

        Ok(())
    }
}

#[derive(Error, Debug)]
pub enum SiteError {
    #[error("non absolute root path")]
    NonAbsoluteRoot,
}
