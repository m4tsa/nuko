use crate::{config::SiteConfig, page::Page};
use anyhow::Result;
use serde_derive::Serialize;
use std::collections::HashMap;

#[derive(Default, Serialize)]
pub struct Sitemap {
    pages: HashMap<String, SitemapEntry>,
}

impl Sitemap {
    pub fn add_page(&mut self, site_config: &SiteConfig, page: &Page) -> Result<()> {
        let path_str: String = page.page_path().to_string_lossy().into();

        self.pages.insert(
            path_str.clone(),
            SitemapEntry {
                permalink: format!(
                    "{}{}{}",
                    site_config.site.base_url,
                    path_str,
                    if path_str.ends_with('/') { "" } else { "/" }
                ),
            },
        );

        Ok(())
    }
}

#[derive(Serialize)]
pub struct SitemapEntry {
    pub permalink: String,
}
