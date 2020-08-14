use crate::ast::*;
use anyhow::Result;
use std::ops::Range;
use thiserror::Error;

pub struct Parser<'a> {
    input: &'a str,
    input_len: usize,
    offset: usize,
    document: OrgDocument,
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a str) -> Parser<'a> {
        Parser {
            input,
            input_len: input.len(),
            offset: 0,
            document: OrgDocument::default(),
        }
    }

    fn next_char(&mut self) -> Option<char> {
        let c = self.peek_char();

        if let Some(c) = c {
            self.offset += c.len_utf8();
        }

        c
    }

    fn prev_char(&self) -> Option<char> {
        if self.offset == 0 {
            return None;
        }

        Some(self.input[(self.offset - 1)..].chars().next().unwrap())
    }

    fn peek_char(&self) -> Option<char> {
        self.input[self.offset..].chars().next()
    }

    fn peek_char_offset(&self, offset: usize) -> Option<char> {
        self.input[self.offset + offset..].chars().next()
    }

    fn continue_until<F>(&mut self, mut map_fn: F) -> usize
    where
        F: FnMut(char) -> bool,
    {
        let mut len = 0;

        while self.peek_char().map(|c| map_fn(c)).unwrap_or(false) {
            self.next_char();
            len += 1;
        }

        len
    }

    fn next_if(&mut self, c: char) -> bool {
        if self.peek_char() == Some(c) {
            self.offset += c.len_utf8();
            true
        } else {
            false
        }
    }

    fn sub_str(&self, start: usize, end: usize) -> Result<&str> {
        self.input
            .get(start..end)
            .ok_or_else(|| ParserError::SubStrOutOfRange { range: start..end }.into())
    }

    pub fn parse_content(&mut self) -> Result<OrgContent> {
        let start_offset = self.offset;
        let prev_char = self.prev_char();

        if prev_char.is_some() && prev_char != Some('\n') {
            panic!("aaa");
        }

        // Headline stars
        if self.continue_until(|c| c == '*') > 0 {
            unimplemented!("headline");
        }
        //
        match (self.peek_char(), self.peek_char_offset(1)) {
            // Comment
            (Some('#'), Some(' ')) => {
                self.continue_until(|c| c != '\n');
                // Eat the newline
                self.next_char().unwrap();
                return Ok(OrgContent::Comment(
                    self.sub_str(start_offset + 2, self.offset - 1)?.into(),
                ));
            }
            // Keyword start
            (Some('#'), Some('+')) => {
                self.next_char().unwrap();
                self.next_char().unwrap();

                self.continue_until(|c| c != '\n');

                let text = self.sub_str(start_offset + 2, self.offset)?;

                if let Some(pos) = text.find(": ") {
                    let key = text[..pos].into();
                    let value = text[pos + 2..].into();

                    // Eat the newline
                    self.next_char().unwrap();

                    return Ok(OrgContent::Keyword(OrgKeyword { key, value }));
                // It's not a keyword then :(
                } else {
                    self.offset = start_offset;
                }
            }
            _ => {}
        }

        let content = self.parse_section_content(start_offset)?;
        Ok(OrgContent::Section(OrgSection {
            headline: None,
            children: content,
        }))
    }

    pub fn parse_section_content(&mut self, start_offset: usize) -> Result<Vec<OrgSectionContent>> {
        let mut section_content = Vec::new();

        self.continue_until(|c| c != '\n');

        // TODO: Handle text effects + elements like date, links and images
        let text = self.sub_str(start_offset, self.offset)?;

        section_content.push(OrgSectionContent::Text(text.into()));

        if self.next_if('\n') {
            section_content.push(OrgSectionContent::Newline);
        }

        Ok(section_content)
    }

    pub fn parse(mut self) -> Result<OrgDocument> {
        while self.offset < self.input_len {
            let content = self.parse_content()?;
            self.document.content.push(content);
        }

        Ok(self.document)
    }
}

#[derive(Error, Debug)]
pub enum ParserError {
    #[error("substr out of range {:?}", range)]
    SubStrOutOfRange { range: Range<usize> },
}

#[cfg(test)]
mod tests {
    use super::{
        OrgContent, OrgDocument, OrgKeyword, OrgSection, OrgSectionContent, Parser, Result,
    };

    fn parse(input: &str) -> Result<OrgDocument> {
        Parser::new(input).parse()
    }

    #[test]
    fn comment() {
        let document = parse("# test\nhello").expect("comment test");

        assert_eq!(
            document.content,
            vec![
                OrgContent::Comment("test".into()),
                OrgContent::Section(OrgSection {
                    children: vec![OrgSectionContent::Text("hello".into())],
                    ..Default::default()
                })
            ]
        )
    }

    #[test]
    fn keyword() {
        let document = parse("#+TITLE: test\nhello").expect("comment test");

        assert_eq!(
            document.content,
            vec![
                OrgContent::Keyword(OrgKeyword {
                    key: "TITLE".into(),
                    value: "test".into()
                }),
                OrgContent::Section(OrgSection {
                    children: vec![OrgSectionContent::Text("hello".into())],
                    ..Default::default()
                })
            ]
        )
    }
}
