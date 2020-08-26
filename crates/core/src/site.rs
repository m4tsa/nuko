use crate::{config::SiteConfig, page::Page, posts::Posts, sitemap::Sitemap, template_fns};
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
    posts: Posts,
    sitemap: Sitemap,
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

            let mut tera = Tera::parse(&format!(
                "{}/templates/**/**.html",
                theme_path.to_string_lossy()
            ))?;

            let robots_path = theme_path.join("templates").join("robots.txt");
            let sitemap_path = theme_path.join("templates").join("sitemap.xml");

            if robots_path.is_file() {
                tera.add_template_file(robots_path, Some("robots.txt"))?;
            }

            if sitemap_path.is_file() {
                tera.add_template_file(sitemap_path, Some("sitemap.xml"))?;
            }

            tera
        } else {
            unimplemented!("builtin theme");
        };

        tera.build_inheritance_chains()?;

        Ok(Site {
            root_path: root_path.into(),
            out_path,
            site_config,
            sitemap: Sitemap::default(),
            tera,
            pages: HashMap::new(),
            posts: Posts::default(),
        })
    }

    pub fn set_baseurl(&mut self, base_url: &str) {
        self.site_config.site.base_url = base_url.into();
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

        let mut post_paths: Vec<PathBuf> = Vec::new();

        for page in &pages {
            if page.ty() == Some("posts") {
                post_paths.push(page.page_path().into());
            }
        }

        for page in pages {
            self.sitemap.add_page(&self.site_config, &page)?;

            if let Some(parent) = page.page_path().parent() {
                if post_paths.contains(&parent.into()) {
                    self.posts.add_page(&page)?;
                }
            }

            self.pages.insert(page.page_path().into(), page);
        }

        self.posts.sort();

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
        self.tera.register_function(
            "get_url",
            template_fns::GetUrl::new(self.site_config.clone(), self.out_path.clone()),
        );

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
        tera_context.insert("sitemap", &self.sitemap);

        self.render_404(&tera_context)?;
        self.render_robots(&tera_context)?;
        self.render_sitemap(&tera_context)?;
        self.merge_static()?;

        Ok(())
    }

    pub fn render_404(&mut self, context: &tera::Context) -> Result<()> {
        let contents = self.render_template("404.html", context)?;
        fs::write(self.out_path.join("404.html"), contents)?;

        Ok(())
    }

    pub fn render_robots(&mut self, context: &tera::Context) -> Result<()> {
        let contents = self.render_template("robots.txt", context)?;
        fs::write(self.out_path.join("robots.txt"), contents)?;

        Ok(())
    }

    pub fn render_sitemap(&mut self, context: &tera::Context) -> Result<()> {
        let contents = self.render_template("sitemap.xml", context)?;
        fs::write(self.out_path.join("sitemap.xml"), contents)?;

        Ok(())
    }

    pub fn render_page(&self, page: &Page) -> Result<()> {
        let mut tera_context = tera::Context::new();

        let (toc, html) = page.render_html(&self.site_config.site.base_url)?;

        tera_context.insert("site_config", &self.site_config);
        tera_context.insert("posts", &self.posts);
        tera_context.insert("page", &page);
        tera_context.insert("document", &html);
        tera_context.insert("toc", &toc);

        let contents =
            self.render_template(page.template().unwrap_or("page.html"), &tera_context)?;

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
            let mut static_dirs = Vec::new();

            let theme_static_dir = self.root_path.join("themes").join(theme).join("static");
            if theme_static_dir.is_dir() {
                static_dirs.push(theme_static_dir);
            }

            let site_static_dir = self.root_path.join("static");
            if site_static_dir.is_dir() {
                static_dirs.push(site_static_dir);
            }

            if !static_dirs.is_empty() {
                let out = self.out_path.join("static");

                for dir in static_dirs {
                    // Workaround for some copying behavior
                    let out = if out.is_dir() {
                        out.join("..")
                    } else {
                        out.clone()
                    };

                    fs_extra::dir::copy(
                        &dir,
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
