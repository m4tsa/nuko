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
    pages: HashMap<PathBuf, Page>,
}

impl Site {
    pub fn new(root_path: &Path, site_config: SiteConfig, out_path: PathBuf) -> Result<Site> {
        if !root_path.is_absolute() {
            return Err(SiteError::NonAbsoluteRoot.into());
        }

        let mut tera = if let Some(theme) = &site_config.site.theme {
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

        tera.build_inheritance_chains()?;

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

        let pages: Vec<Page> = glob(&format!("{}/**/*.org", content_dir.to_string_lossy()))?
            .filter_map(|p| p.ok())
            .filter(|e| {
                !e.as_path()
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .starts_with('.')
            })
            .map(|page_path| Page::read_file(&self.root_path, page_path, &self.site_config))
            .collect::<Result<Vec<Page>>>()?;

        for page in pages {
            self.pages.insert(page.page_path().into(), page);
        }

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

        // Render pages
        for (_, page) in &self.pages {
            self.render_page(page)?;
        }

        // Render extras
        let mut tera_context = tera::Context::new();
        tera_context.insert("site_config", &self.site_config);

        self.render_404(&tera_context)?;
        self.merge_static()?;

        Ok(())
    }

    pub fn render_404(&mut self, context: &tera::Context) -> Result<()> {
        let contents = self.render_template("404.html", context)?;
        fs::write(self.out_path.join("404.html"), contents)?;

        Ok(())
    }

    pub fn render_page(&self, page: &Page) -> Result<()> {
        let mut tera_context = tera::Context::new();

        let (toc, html) = page.render_html()?;

        tera_context.insert("site_config", &self.site_config);
        tera_context.insert("page", &page);
        tera_context.insert("document", &html);
        tera_context.insert("toc", &toc);

        let contents = self.render_template("page.html", &tera_context)?;

        let out_path = self.out_path.join(page.page_path().strip_prefix("/")?);

        fs::create_dir_all(&out_path)?;
        fs::write(out_path.join("index.html"), contents)?;

        Ok(())
    }

    fn render_template(&self, name: &str, context: &tera::Context) -> Result<String> {
        self.tera.render(name, context).map_err(|e| e.into())
    }

    fn merge_static(&self) -> Result<()> {
        if let Some(theme) = &self.site_config.site.theme {
            let static_dir = self.root_path.join("themes").join(theme).join("static");

            if static_dir.is_dir() {
                let out = self.out_path.join("static");

                fs_extra::copy_items(
                    &vec![&static_dir],
                    &out,
                    &fs_extra::dir::CopyOptions {
                        overwrite: false,
                        skip_exist: false,
                        buffer_size: 64 * 1024,
                        depth: 0,
                        copy_inside: true,
                    },
                )?;
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
    #[error("error compiling tera template at path \"{0}\": \n{1}")]
    Tera(String, String),
}
