use crate::page::Page;
use anyhow::Result;
use chrono::NaiveDate;
use serde_derive::Serialize;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Serialize)]
pub struct Post {
    title: String,
    description: String,
    page_path: PathBuf,
    date: NaiveDate,
    date_updated: Option<NaiveDate>,
}

#[derive(Default, Serialize)]
pub struct Posts {
    posts: Vec<Post>,
}

impl Posts {
    pub fn add_page(&mut self, page: &Page) -> Result<()> {
        let page_path = page.page_path();

        let post = Post {
            title: page
                .title()
                .ok_or_else(|| PostsError::MissingTitle(page_path.into()))?
                .into(),
            description: page
                .description()
                .ok_or_else(|| PostsError::MissingDescription(page_path.into()))?
                .into(),
            page_path: page_path.into(),
            date: page
                .date()
                .ok_or_else(|| PostsError::MissingDate(page_path.into()))?
                .clone(),
            date_updated: page.date_updated().map(|d| d.clone()),
        };

        self.posts.push(post);

        Ok(())
    }

    pub fn sort(&mut self) {
        self.posts.sort_by(|a, b| b.date.cmp(&a.date))
    }
}

#[derive(Error, Debug)]
pub enum PostsError {
    #[error("post at \"{0:?}\" is missing the title keyword")]
    MissingTitle(PathBuf),
    #[error("post at \"{0:?}\" is missing the description keyword")]
    MissingDescription(PathBuf),
    #[error("post at \"{0:?}\" is missing the date keyword")]
    MissingDate(PathBuf),
}
