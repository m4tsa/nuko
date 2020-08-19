use crate::{config::SiteConfig, org_emitter::emit_document};
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
    title: Option<String>,
    description: Option<String>,
    page_path: PathBuf,
}

impl Page {
    pub fn parse(page_path: PathBuf, text: String, _config: &SiteConfig) -> Result<Page> {
        let document = Parser::new(&text).parse()?;

        let title = document.get_keyword("TITLE").map(|t| t.to_string());
        let description = document.get_keyword("DESCRIPTION").map(|t| t.to_string());

        Ok(Page {
            document,
            title,
            description,
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

    pub fn render_html(&self) -> Result<String> {
        emit_document(&self.document)
    }

    pub fn document(&self) -> &OrgDocument {
        &self.document
    }

    pub fn page_path(&self) -> &Path {
        &self.page_path
    }
}
