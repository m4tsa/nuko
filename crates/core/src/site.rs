use crate::{config::SiteConfig, page::Page};
use anyhow::Result;
use glob::glob;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};
use tera::Tera;
use thiserror::Error;

pub struct Site {
    site_config: SiteConfig,
    out_path: PathBuf,
    root_path: PathBuf,
    tera: Tera,
    pages: HashMap<String, Page>,
}

impl Site {
    pub fn new(root_path: &Path, site_config: SiteConfig, out_path: PathBuf) -> Result<Site> {
        if !root_path.is_absolute() {
            return Err(SiteError::NonAbsoluteRoot.into());
        }

        let tera = if let Some(theme) = &site_config.site.theme {
            let theme_path = root_path.join("themes").join(theme);

            if !theme_path.is_dir() {
                return Err(SiteError::SiteIsMissingTheme(theme.into()).into());
            }

            Tera::parse(&format!(
                "{}/templates/**/**.html",
                theme_path.to_string_lossy()
            ))?
        } else {
            unimplemented!("builtin theme");
        };

        Ok(Site {
            root_path: root_path.into(),
            out_path,
            site_config,
            tera,
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
    #[error("the site is missing the specified theme: \"{}\"", _0)]
    SiteIsMissingTheme(String),
}
