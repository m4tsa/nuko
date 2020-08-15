use crate::ast::*;
use anyhow::Result;
use fancy_regex::Regex;
use lazy_static::lazy_static;
use std::ops::Range;
use thiserror::Error;

pub struct Parser<'a> {
    input: &'a str,
    input_len: usize,
    offset: usize,
    document: OrgDocument,
    headline: Option<OrgHeadline>,
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a str) -> Parser<'a> {
        Parser {
            input,
            input_len: input.len(),
            offset: 0,
            document: OrgDocument::default(),
            headline: None,
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

    fn get_last_section(&mut self) -> &mut OrgSection {
        let content = &mut self.document.content;

        if let Some(i) = content.iter().enumerate().rev().find_map(|(i, content)| {
            if let OrgContent::Section(_) = content {
                Some(i)
            } else {
                None
            }
        }) {
            if let OrgContent::Section(section) = &mut content[i] {
                return section;
            } else {
                unreachable!();
            }
        } else {
            content.push(OrgContent::Section(OrgSection::default()));
            content
                .last_mut()
                .map(|c| {
                    if let OrgContent::Section(section) = c {
                        section
                    } else {
                        unreachable!()
                    }
                })
                .unwrap()
        }
    }

    fn parse_content(&mut self) -> Result<()> {
        let start_offset = self.offset;
        let prev_char = self.prev_char();

        if prev_char.is_some() && prev_char != Some('\n') {
            return Err(ParserError::BadStart.into());
        }

        // Headline stars
        let stars = self.continue_until(|c| c == '*');

        if stars > 0 {
            // Headlines needs a space after the stars
            if self.next_char() == Some(' ') {
                // Check if there is a previous unused headline
                if let Some(headline) = self.headline.take() {
                    self.document.content.push(OrgContent::Section(OrgSection {
                        headline: Some(headline),
                        children: vec![],
                    }));
                }

                let headline_text_start = self.offset;

                self.continue_until(|c| c != '\n');

                let mut keyword = None;

                let headline_text = self.sub_str(headline_text_start, self.offset)?;
                let mut content_start = headline_text_start;

                let mut splitted = headline_text.split_ascii_whitespace();

                if let Some(first_word) = splitted.next() {
                    let is_keyword = match first_word {
                        "TODO" | "DONE" => true,
                        _ => false,
                    };

                    if is_keyword {
                        keyword = Some(first_word.into());
                        content_start += first_word.len() + ' '.len_utf8();
                    }
                }

                let content = self.parse_section_content(content_start)?;

                self.headline = Some(OrgHeadline {
                    level: stars as u8,
                    keyword,
                    content,
                    ..Default::default()
                });
            } else {
                self.offset = start_offset;
            }
        }

        // Peek for other special start of line things
        match (self.peek_char(), self.peek_char_offset(1)) {
            // Comment
            (Some('#'), Some(' ')) => {
                self.continue_until(|c| c != '\n');
                // Eat the newline
                self.next_char().unwrap();
                self.document.content.push(OrgContent::Comment(
                    self.sub_str(start_offset + 2, self.offset - 1)?.into(),
                ));

                return Ok(());
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

                    self.document
                        .content
                        .push(OrgContent::Keyword(OrgKeyword { key, value }));

                    return Ok(());
                // It's not a keyword then :(
                } else {
                    self.offset = start_offset;
                }
            }
            _ => {}
        }

        let mut content = self.parse_section_content(self.offset)?;

        // Create a new section on new headline, otherwise append
        if let Some(headline) = self.headline.take() {
            self.document.content.push(OrgContent::Section(OrgSection {
                headline: Some(headline),
                children: content,
            }));
        } else {
            let section = self.get_last_section();

            // Add a newline if there is a newline in the section
            if !section.children.is_empty() {
                section.children.push(OrgSectionContent::Newline);
            }

            section.children.append(&mut content);
        }

        Ok(())
    }

    fn parse_section_content(&mut self, start_offset: usize) -> Result<Vec<OrgSectionContent>> {
        lazy_static! {
            pub static ref EMPHASIS_REGEX: Regex =
                Regex::new(r"(?:^|[ ])([\*|\/|\_|\=|\~|\+])([^\*]+?)\1(?:[ ]|$)").unwrap();
        }

        fn parse_section(text: &str) -> Result<Vec<OrgSectionContent>> {
            if text.is_empty() {
                return Ok(Vec::with_capacity(0));
            }

            match EMPHASIS_REGEX.captures(text) {
                Ok(captures) => {
                    if let Some(captures) = captures {
                        let mut content = Vec::new();

                        let capture = captures.get(0).unwrap();
                        let capture_str = capture.as_str();
                        let (mut start, mut end) = (capture.start(), capture.end());

                        // The regex can start or stop with a space, correct this no since the no match in the capture group does not change the actual capture range
                        if capture_str.starts_with(' ') {
                            start += 1;
                        }

                        if capture_str.ends_with(' ') {
                            end -= 1;
                        }

                        if start > 0 {
                            content.push(OrgSectionContent::Text(text[..start].into()));
                        }

                        let emphasis_ty = captures.get(1).unwrap().as_str();
                        let inner = parse_section(captures.get(2).unwrap().as_str())?;

                        let section_content = match emphasis_ty {
                            "*" => OrgSectionContent::Bold(inner),
                            "/" => OrgSectionContent::Italic(inner),
                            "_" => OrgSectionContent::Underlined(inner),
                            "=" => OrgSectionContent::Verbatim(inner),
                            "~" => OrgSectionContent::Code(inner),
                            "+" => OrgSectionContent::Strikethrough(inner),
                            _ => unreachable!(),
                        };

                        content.push(section_content);

                        content.append(&mut parse_section(&text[end..])?);

                        Ok(content)
                    } else {
                        Ok(vec![OrgSectionContent::Text(text.into())])
                    }
                }
                Err(err) => panic!(err),
            }
        }

        self.continue_until(|c| c != '\n');

        // TODO: Handle text effects + elements like date, links and images
        let text = self.sub_str(start_offset, self.offset)?;
        let content = parse_section(text)?;

        self.next_if('\n');

        Ok(content)
    }

    pub fn parse(mut self) -> Result<OrgDocument> {
        while self.offset < self.input_len {
            self.parse_content()?;
        }

        Ok(self.document)
    }
}

#[derive(Error, Debug)]
pub enum ParserError {
    #[error("substr out of range {:?}", range)]
    SubStrOutOfRange { range: Range<usize> },
    #[error("org-mode parser did not start at column 0")]
    BadStart,
}

#[cfg(test)]
mod tests {
    use super::{
        OrgContent, OrgDocument, OrgHeadline, OrgKeyword, OrgSection, OrgSectionContent, Parser,
        Result,
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
    fn emphasis() {
        let document = parse("hello *there* /nice +day+ today/").expect("comment test");

        assert_eq!(
            document.content,
            vec![OrgContent::Section(OrgSection {
                headline: None,
                children: vec![
                    OrgSectionContent::Text("hello ".into()),
                    OrgSectionContent::Bold(vec![OrgSectionContent::Text("there".into())]),
                    OrgSectionContent::Text(" ".into()),
                    OrgSectionContent::Italic(vec![
                        OrgSectionContent::Text("nice ".into()),
                        OrgSectionContent::Strikethrough(vec![OrgSectionContent::Text(
                            "day".into()
                        )]),
                        OrgSectionContent::Text(" today".into()),
                    ])
                ],
                ..Default::default()
            })]
        );
    }

    #[test]
    fn headline() {
        let document = parse("* test\nhello").expect("comment test");

        assert_eq!(
            document.content,
            vec![OrgContent::Section(OrgSection {
                headline: Some(OrgHeadline {
                    level: 1,
                    content: vec![OrgSectionContent::Text("test".into())],
                    ..OrgHeadline::default()
                }),
                children: vec![OrgSectionContent::Text("hello".into())],
                ..Default::default()
            })]
        );

        let document = parse("* TODO test\nhello").expect("comment test");

        assert_eq!(
            document.content,
            vec![OrgContent::Section(OrgSection {
                headline: Some(OrgHeadline {
                    level: 1,
                    keyword: Some("TODO".into()),
                    content: vec![OrgSectionContent::Text("test".into())],
                    ..OrgHeadline::default()
                }),
                children: vec![OrgSectionContent::Text("hello".into())],
                ..Default::default()
            })]
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

    #[test]
    fn newline() {
        let document = parse("** test\nhello\nthere").expect("comment test");

        assert_eq!(
            document.content,
            vec![OrgContent::Section(OrgSection {
                headline: Some(OrgHeadline {
                    level: 2,
                    content: vec![OrgSectionContent::Text("test".into())],
                    ..OrgHeadline::default()
                }),
                children: vec![
                    OrgSectionContent::Text("hello".into()),
                    OrgSectionContent::Newline,
                    OrgSectionContent::Text("there".into())
                ],
                ..Default::default()
            })]
        )
    }
}
