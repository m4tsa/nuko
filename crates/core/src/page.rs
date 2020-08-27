use crate::{config::SiteConfig, org_emitter::emit_document, toc::Toc};
use anyhow::Result;
use chrono::NaiveDate;
use nuko_org_parser::{ast::OrgDocument, parser::Parser};
use serde_derive::Serialize;
use std::{
    fs,
    path::{Path, PathBuf},
};
use thiserror::Error;

#[derive(Serialize)]
pub struct Page {
    document: OrgDocument,
    title: Option<String>,
    ty: Option<String>,
    template: Option<String>,
    description: Option<String>,
    page_path: PathBuf,
    date: Option<NaiveDate>,
    date_updated: Option<NaiveDate>,
}

impl Page {
    pub fn parse(page_path: PathBuf, text: String, _config: &SiteConfig) -> Result<Page> {
        let document = Parser::new(&text).parse()?;

        let title = document.get_keyword("TITLE").map(|t| t.to_string());
        let ty = document.get_keyword("TYPE").map(|t| t.to_string());
        let template = document.get_keyword("TEMPLATE").map(|t| t.to_string());
        let description = document.get_keyword("DESCRIPTION").map(|t| t.to_string());

        let (date, date_updated) = if let Some(value) = document.get_keyword("DATE") {
            if !value.starts_with('<') {
                return Err(PageError::InvalidDateField(
                    "Missing timestamp".into(),
                    value.to_string(),
                )
                .into());
            }

            let end = value.find('>').ok_or_else(|| {
                PageError::InvalidDateField("Unterminated timestamp".into(), value.to_string())
            })?;

            let date = NaiveDate::parse_from_str(&value[1..end], "%Y-%m-%d %a")?;

            let rest = &value[(end + 1)..];

            let mut edited = None;

            if !rest.is_empty() {
                if !rest.starts_with("---") {
                    return Err(PageError::InvalidDateField(
                        "Wrong sequence between dates".into(),
                        value.to_string(),
                    )
                    .into());
                }

                let rest = &value[(end + 4)..];

                if !rest.starts_with('<') {
                    return Err(PageError::InvalidDateField(
                        "Missing timestamp".into(),
                        value.to_string(),
                    )
                    .into());
                }

                let end = rest.find('>').ok_or_else(|| {
                    PageError::InvalidDateField("Unterminated timestamp".into(), value.to_string())
                })?;

                edited = Some(NaiveDate::parse_from_str(&rest[1..end], "%Y-%m-%d %a")?);
            }

            (Some(date), edited)
        } else {
            (None, None)
        };

        Ok(Page {
            document,
            title,
            ty,
            template,
            description,
            page_path,
            date,
            date_updated,
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

    pub fn date(&self) -> Option<&NaiveDate> {
        self.date.as_ref()
    }

    pub fn date_updated(&self) -> Option<&NaiveDate> {
        self.date_updated.as_ref()
    }

    pub fn page_path(&self) -> &Path {
        &self.page_path
    }
}

#[derive(Error, Debug)]
pub enum PageError {
    #[error("invalid date field: {0} \"{1}\"")]
    InvalidDateField(String, String),
}
