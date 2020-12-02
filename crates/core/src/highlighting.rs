use anyhow::Result;
use std::path::Path;
use syntect::{
    highlighting::{Theme, ThemeSet},
    parsing::{SyntaxReference, SyntaxSet},
};
use thiserror::Error;

lazy_static! {
    static ref DEFAULT_THEME: String = String::from("base16-ocean.dark");
}

pub struct Highlighting {
    syntax: Option<String>,
    syntax_set: SyntaxSet,
    theme_set: ThemeSet,
}

impl Highlighting {
    pub fn new(root_path: &Path, theme: Option<String>) -> Result<Highlighting> {
        let mut syntax_set = SyntaxSet::load_defaults_newlines();
        let mut theme_set = ThemeSet::load_defaults();

        let highlighting_path = root_path.join("highlighting");

        if highlighting_path.is_dir() {
            let highlighting_syntaxes_path = highlighting_path.join("syntaxes");

            if highlighting_syntaxes_path.is_dir() {
                let mut builder = syntax_set.into_builder();
                builder.add_from_folder(&highlighting_syntaxes_path, true)?;
                syntax_set = builder.build();
            }

            let highlighting_themes_path = highlighting_path.join("themes");

            if highlighting_themes_path.is_dir() {
                theme_set.add_from_folder(&highlighting_themes_path)?;
            }
        }

        Ok(Highlighting {
            syntax: theme,
            syntax_set,
            theme_set,
        })
    }

    pub fn find_syntax_by_name(&self, name: &str) -> Result<&SyntaxReference> {
        self.syntax_set
            .find_syntax_by_name(name)
            .ok_or_else(|| HighlightingError::UnknownSyntax(name.into()).into())
    }

    pub fn theme(&self) -> Result<&Theme> {
        let name = self.syntax.as_ref().unwrap_or(&DEFAULT_THEME);
        self.theme_set
            .themes
            .get(name)
            .ok_or_else(|| HighlightingError::UnknownSyntaxTheme(name.into()).into())
    }

    pub fn syntaxes(&self) -> &SyntaxSet {
        &self.syntax_set
    }
}

#[derive(Error, Debug)]
pub enum HighlightingError {
    #[error("cannot find syntax highlighting for language \"{0}\"")]
    UnknownSyntax(String),
    #[error("cannot find syntax theme \"{0}\"")]
    UnknownSyntaxTheme(String),
}
