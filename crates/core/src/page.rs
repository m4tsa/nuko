use crate::{config::SiteConfig, org_emitter::emit_document, toc::Toc};
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
    ty: Option<String>,
    template: Option<String>,
    description: Option<String>,
    page_path: PathBuf,
}

impl Page {
    pub fn parse(page_path: PathBuf, text: String, _config: &SiteConfig) -> Result<Page> {
        let document = Parser::new(&text).parse()?;

        let title = document.get_keyword("TITLE").map(|t| t.to_string());
        let ty = document.get_keyword("TYPE").map(|t| t.to_string());
        let template = document.get_keyword("TEMPLATE").map(|t| t.to_string());
        let description = document.get_keyword("DESCRIPTION").map(|t| t.to_string());

        Ok(Page {
            document,
            title,
            ty,
            template,
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

    pub fn render_html(&self, base_url: &str) -> Result<(Toc, String)> {
        emit_document(&self.document, base_url)
    }

    pub fn document(&self) -> &OrgDocument {
        &self.document
    }

    pub fn title(&self) -> Option<&str> {
        self.title.as_ref().map(|s| s.as_str())
    }

    pub fn description(&self) -> Option<&str> {
        self.description.as_ref().map(|s| s.as_str())
    }

    pub fn ty(&self) -> Option<&str> {
        self.ty.as_ref().map(|s| s.as_str())
    }

    pub fn template(&self) -> Option<&str> {
        self.template.as_ref().map(|s| s.as_str())
    }

    pub fn page_path(&self) -> &Path {
        &self.page_path
    }
}
