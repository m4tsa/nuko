use crate::config::SiteConfig;
use anyhow::Result;
use std::{fs, path::Path};

pub struct Page {}

impl Page {
    pub fn parse(root_path: &Path, path: &Path, text: String, config: &SiteConfig) -> Result<Page> {
        unimplemented!()
    }

    pub fn read_file(root_path: &Path, path: &Path, config: &SiteConfig) -> Result<Page> {
        let text = fs::read_to_string(path)?;

        Page::parse(root_path, path, text, config)
    }
}
