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

        let date = NaiveDate::from_ymd(2020, 1, 1);

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
            date,
            date_updated: None,
        };

        self.posts.push(post);

        Ok(())
    }

    pub fn sort(&mut self) {
        self.posts.sort_by(|a, b| a.date.cmp(&b.date))
    }
}

#[derive(Error, Debug)]
pub enum PostsError {
    #[error("post at \"{0:?}\" is missing the title keyword")]
    MissingTitle(PathBuf),
    #[error("post at \"{0:?}\" is missing the description keyword")]
    MissingDescription(PathBuf),
}