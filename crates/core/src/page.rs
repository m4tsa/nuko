use crate::config::SiteConfig;
use anyhow::Result;
use nuko_org_parser::{ast::OrgDocument, parser::Parser};
use serde_derive::Serialize;
use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Serialize)]
pub struct Page {
    document: OrgDocument,
    page_path: PathBuf,
}

impl Page {
    pub fn parse(page_path: PathBuf, text: String, _config: &SiteConfig) -> Result<Page> {
        let document = Parser::new(&text).parse()?;

        Ok(Page {
            document,
            page_path,
        })
    }

    pub fn read_file(root_path: &Path, path: PathBuf, config: &SiteConfig) -> Result<Page> {
        let text = fs::read_to_string(&path)?;

        let content_path = root_path.join("content");

        let is_root = path
            .file_name()
            .unwrap()
            .to_string_lossy()
            .starts_with("_index");

        // Create absolute page paths
        let page_path = if is_root {
            Path::new("/").join(
                path.strip_prefix(&content_path)
                    .unwrap()
                    .parent()
                    .unwrap_or(Path::new("/")),
            )
        } else {
            Path::new("/").join(path.strip_prefix(&content_path).unwrap().with_extension(""))
        };

        Page::parse(page_path, text, config)
    }

    pub fn document(&self) -> &OrgDocument {
        &self.document
    }

    pub fn page_path(&self) -> &Path {
        &self.page_path
    }
}
