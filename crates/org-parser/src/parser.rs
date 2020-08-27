use crate::ast::*;
use anyhow::Result;
use fancy_regex::Regex as FancyRegex;
use lazy_static::lazy_static;
use regex::Regex;
use std::ops::Range;
use thiserror::Error;

// TODO: Clean everything up, this was written by looking at the org syntax page with no plans at all.
// TODO: Remove duplicated code

pub struct Parser<'a> {
    input: &'a str,
    input_len: usize,
    offset: usize,
    document: OrgDocument,
    headline: Option<OrgHeadline>,
}

lazy_static! {
    pub static ref EMPHASIS_REGEX: FancyRegex =
        FancyRegex::new(r"(?:^|[ ])([\*|\/|\_|\=|\~|\+])([^\*]+?)?\1").unwrap();
    pub static ref EXTRAS_REGEX: Regex =
        Regex::new(r"\[\[(.+?)\]\[(.+?)\]\]|\[fn:(|.+?)?:(.+?[^\]])\]((?:[^\[\]])|$)").unwrap();
}

fn parse_content(
    mut section: Option<&mut OrgSection>,
    text: &str,
) -> Result<Vec<OrgSectionContent>> {
    if text.is_empty() {
        return Ok(Vec::with_capacity(0));
    }

    // Check if the line starts with an unordered list
    if let Some(list_start) = text.find("- ") {
        let start_text = &text[..list_start];

        if start_text.chars().all(|f| f.is_ascii_whitespace()) {
            let list_text = &text[(list_start + 2)..];

            // TODO: Support new list types
            let list_ty = OrgListType::Bullet;

            let list_entry_content = parse_content(section.as_deref_mut(), list_text)?;

            if let Some(section) = section.as_deref_mut() {
                let last_entry = section.children.last_mut();

                match last_entry {
                    Some(OrgSectionContent::List(list)) => {
                        if list.ty == list_ty {
                            list.values.push(OrgListValue::Content(list_entry_content));

                            return Ok(Vec::with_capacity(0));
                        }
                    }
                    _ => {}
                }
            }

            return Ok(vec![OrgSectionContent::List(OrgListEntry {
                ty: list_ty,
                values: vec![OrgListValue::Content(list_entry_content)],
            })]);
        }
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

                if let Some(captures) = EXTRAS_REGEX.captures(text) {
                    let capture = captures.get(0).unwrap();

                    if capture.start() < start {
                        let mut content = Vec::new();

                        let mut end = capture.end();

                        if capture.start() > 0 {
                            content.push(OrgSectionContent::Text(text[..capture.start()].into()));
                        }

                        if let (Some(link), Some(label)) = (
                            captures.get(1).map(|m| m.as_str()),
                            captures.get(2).map(|m| m.as_str()),
                        ) {
                            content.push(OrgSectionContent::Link {
                                link: link.into(),
                                label: parse_content(section.as_deref_mut(), label)?,
                            });
                        } else if let (Some(fn_name), Some(fn_content), Some(fn_extra)) = (
                            captures.get(3).map(|m| m.as_str()),
                            captures.get(4).map(|m| m.as_str()),
                            captures.get(5).map(|m| m.as_str()),
                        ) {
                            content.push(OrgSectionContent::Footnote {
                                name: if fn_name.is_empty() {
                                    None
                                } else {
                                    Some(fn_name.into())
                                },
                                content: parse_content(section.as_deref_mut(), fn_content)?,
                            });

                            end -= fn_extra.len();
                        } else {
                            unreachable!();
                        }

                        content.append(&mut parse_content(section, &text[end..])?);

                        return Ok(content);
                    }
                }

                let emphasis_ty = captures.get(1).unwrap().as_str();
                let inner =
                    parse_content(section.as_deref_mut(), captures.get(2).unwrap().as_str())?;

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

                content.append(&mut parse_content(section, &text[end..])?);

                Ok(content)
            } else {
                if let Some(captures) = EXTRAS_REGEX.captures(text) {
                    let capture = captures.get(0).unwrap();
                    let mut content = Vec::new();

                    let (start, mut end) = (capture.start(), capture.end());

                    if capture.start() > 0 {
                        content.push(OrgSectionContent::Text(text[..start].into()));
                    }

                    if let (Some(link), Some(label)) = (
                        captures.get(1).map(|m| m.as_str()),
                        captures.get(2).map(|m| m.as_str()),
                    ) {
                        content.push(OrgSectionContent::Link {
                            link: link.into(),
                            label: parse_content(section.as_deref_mut(), label)?,
                        });
                    } else if let (Some(fn_name), Some(fn_content), Some(fn_extra)) = (
                        captures.get(3).map(|m| m.as_str()),
                        captures.get(4).map(|m| m.as_str()),
                        captures.get(5).map(|m| m.as_str()),
                    ) {
                        content.push(OrgSectionContent::Footnote {
                            name: if fn_name.is_empty() {
                                None
                            } else {
                                Some(fn_name.into())
                            },
                            content: parse_content(section.as_deref_mut(), fn_content)?,
                        });

                        end -= fn_extra.len();
                    } else {
                        unreachable!();
                    }

                    content.append(&mut parse_content(section, &text[end..])?);

                    return Ok(content);
                }

                Ok(vec![OrgSectionContent::Text(text.into())])
            }
        }
        Err(err) => panic!(err),
    }
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
        if self.offset + offset > self.input_len {
            return None;
        }

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

                self.continue_until(|c| c != '\n');

                let text = self.sub_str(content_start, self.offset)?;
                let content = parse_content(None, text)?;

                self.next_if('\n');

                self.headline = Some(OrgHeadline {
                    level: stars as u8,
                    keyword,
                    content,
                    ..Default::default()
                });

                return Ok(());
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
                    if text == "BEGIN_HTML" {
                        // Eat the newline
                        self.next_char().unwrap();

                        let mut start_offset = self.offset;

                        let mut html = String::new();

                        while self.peek_char().is_some() {
                            self.continue_until(|c| c != '\n');

                            let text = self.sub_str(start_offset, self.offset)?.to_string();
                            // Eat the newline if it exists
                            self.next_char();

                            if text == "#+END_HTML" {
                                break;
                            } else {
                                html.push_str(&text);
                            }

                            start_offset = self.offset;
                        }

                        self.get_last_section()
                            .children
                            .push(OrgSectionContent::Html(html));

                        return Ok(());
                    } else {
                        self.offset = start_offset;
                    }
                }
            }
            _ => {}
        }

        if let Some(headline) = self.headline.take() {
            self.document.content.push(OrgContent::Section(OrgSection {
                headline: Some(headline),
                children: vec![],
            }));
        }

        self.continue_until(|c| c != '\n');

        let text = self.sub_str(start_offset, self.offset)?.to_string();
        self.next_if('\n');

        let section = self.get_last_section();
        let mut content = parse_content(Some(section), &text)?;

        if !section.children.is_empty() {
            if let Some(OrgSectionContent::List(_)) = section.children.last() {
                // Do nothing
            } else {
                section.children.push(OrgSectionContent::Newline);
            }
        }

        // Create a new section on new headline, otherwise append
        section.children.append(&mut content);

        Ok(())
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
        let document = parse("hello *there* /nice +day+ today/").expect("emphasis test");

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
        let document = parse("* test\nhello").expect("headline test");

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

        let document = parse("* TODO test\nhello").expect("headline test");

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
        let document = parse("#+TITLE: test\nhello").expect("keyword test");

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
    fn links() {
        let document = parse("[[https://example.com][example]]").expect("links test");

        assert_eq!(
            document.content,
            vec![OrgContent::Section(OrgSection {
                headline: None,
                children: vec![OrgSectionContent::Link {
                    link: "https://example.com".into(),
                    label: vec![OrgSectionContent::Text("example".into())]
                }]
            })]
        )
    }

    #[test]
    fn newline() {
        let document = parse("** test\nhello\nthere").expect("newline test");

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
