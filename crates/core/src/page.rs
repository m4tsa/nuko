use crate::{config::SiteConfig, highlighting::Highlighting, org_emitter::emit_document, toc::Toc};
use anyhow::Result;
use chrono::NaiveDate;
use orgize::Org;
use serde_derive::Serialize;
use std::{
    fs,
    path::{Path, PathBuf},
};
use thiserror::Error;

#[derive(Serialize)]
pub struct Page {
    document: Org<'static>,
    title: Option<String>,
    ty: Option<String>,
    template: Option<String>,
    description: Option<String>,
    page_path: PathBuf,
    date: Option<NaiveDate>,
    date_updated: Option<NaiveDate>,
    tags: Vec<String>,
}

fn get_keyword(document: &Org, keyword: &str) -> Option<String> {
    for keyword_entry in document.keywords() {
        if keyword_entry.key == keyword {
            return Some(keyword_entry.value.to_string());
        }
    }

    None
}

impl Page {
    pub fn parse(page_path: PathBuf, text: String, _config: &SiteConfig) -> Result<Page> {
        let document = Org::parse(Box::leak(text.into_boxed_str()));

        let title = get_keyword(&document, "TITLE").map(|t| t.to_string());
        let ty = get_keyword(&document, "TYPE").map(|t| t.to_string());
        let template = get_keyword(&document, "TEMPLATE").map(|t| t.to_string());
        let description = get_keyword(&document, "DESCRIPTION").map(|t| t.to_string());

        let (date, date_updated) = if let Some(value) = get_keyword(&document, "DATE") {
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

        let mut tags = Vec::new();

        if let Some(tags_value) = get_keyword(&document, "TAGS") {
            for tag in tags_value.split(' ') {
                if tag.chars().any(|c| !c.is_ascii_lowercase()) {
                    return Err(PageError::InvalidTag(tag.into()).into());
                }

                tags.push(tag.into());
            }
        }

        Ok(Page {
            document,
            title,
            ty,
            template,
            description,
            page_path,
            date,
            date_updated,
            tags,
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

    pub fn render_html(
        &self,
        base_url: &str,
        highlighting: &Highlighting,
        highlighting_dark: Option<&Highlighting>,
    ) -> Result<(Toc, String)> {
        emit_document(&self.document, base_url, highlighting, highlighting_dark)
    }

    pub fn document(&self) -> &Org {
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

    pub fn tags(&self) -> &[String] {
        &self.tags
    }
}

#[derive(Error, Debug)]
pub enum PageError {
    #[error("invalid date field: {0} \"{1}\"")]
    InvalidDateField(String, String),
    #[error("invalid tag: \"{0}\"")]
    InvalidTag(String),
}
