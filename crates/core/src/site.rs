use crate::{config::SiteConfig, page::Page};
use anyhow::Result;
use glob::glob;
use std::{
    collections::HashMap,
    fs,
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

    pub fn build_scss(&mut self, path: &Path) -> Result<()> {
        let scss_paths: Vec<PathBuf> = glob(&format!("{}/**/*.scss", path.to_string_lossy()))?
            .filter_map(|p| p.ok())
            .collect();

        for scss_path in scss_paths {
            if !scss_path.is_file() {
                continue;
            }

            let stripped_path = scss_path.strip_prefix(path).unwrap();
            let out_path = self.out_path.join(stripped_path).with_extension("css");

            let css_output = sass_rs::compile_file(
                &scss_path,
                sass_rs::Options {
                    output_style: sass_rs::OutputStyle::Compressed,
                    indented_syntax: false,
                    ..Default::default()
                },
            )
            .map_err(|e| SiteError::Scss(scss_path.to_string_lossy().into(), e))?;

            fs::write(out_path, css_output)?;
        }

        Ok(())
    }

    pub fn build(&mut self) -> Result<()> {
        // Delete the previous out path if exists
        if self.out_path.exists() {
            fs::remove_dir_all(&self.out_path)?;
        }

        // Create the out path
        fs::create_dir_all(&self.out_path)?;

        // Build the themes styles
        if let Some(theme) = &self.site_config.site.theme {
            let scss_path = self.root_path.join("themes").join(theme).join("scss");

            if scss_path.is_dir() {
                self.build_scss(&scss_path)?;
            }
        }

        Ok(())
    }
}

#[derive(Error, Debug)]
pub enum SiteError {
    #[error("non absolute root path")]
    NonAbsoluteRoot,
    #[error("the site is missing the specified theme: \"{}\"", _0)]
    SiteIsMissingTheme(String),
    #[error("error compiling scss at path \"{0}\": \n{1}")]
    Scss(String, String),
}
