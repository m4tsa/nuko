use crate::{highlighting::Highlighting, toc::Toc};
use anyhow::Result;
use orgize::{
    elements::{Element, Link},
    Event, Org,
};
use syntect::{
    easy::HighlightLines,
    html::{styled_line_to_highlighted_html, IncludeBackground},
};
use thiserror::Error;

#[derive(Default)]
pub struct EmitData {
    toc: Toc,
    footnotes: Vec<String>,
}

fn link_to_html(base_url: &str, link: &Link) -> String {
    let (href, extra) = if link.path.starts_with("/") {
        (format!("{}{}", base_url, link.path), "")
    } else {
        (link.path.to_string(), " rel=\"noreferrer noopener\"")
    };

    format!(
        "<a href=\"{}\"{}>{}",
        href,
        extra,
        tera::escape_html(&link.desc.clone().unwrap_or_default())
    )
}

fn emit_element_start(
    out: &mut String,
    base_url: &str,
    data: &mut EmitData,
    element: &Element,
    highlighting: &Highlighting,
) -> Result<()> {
    match element {
        Element::SpecialBlock(_special_block) => {}
        Element::QuoteBlock(_quote_block) => {}
        Element::CenterBlock(_center_block) => {}
        Element::VerseBlock(_verse_block) => {}
        Element::CommentBlock(_comment_block) => {}
        Element::ExampleBlock(_example_block) => {}
        Element::ExportBlock(export_block) => {
            // Emit raw html
            if export_block.data.to_ascii_lowercase() == "html" {
                out.push_str(&export_block.contents);
            }
        }
        Element::SourceBlock(source_block) => {
            if source_block.language.is_empty() {
                out.push_str(&format!(
                    "<pre class=code>{}</pre>",
                    tera::escape_html(&*source_block.contents)
                ));
            } else {
                let syntax = highlighting.find_syntax_by_name(&*source_block.language)?;
                let mut syntax_highlighter = HighlightLines::new(syntax, highlighting.theme()?);
                let regions =
                    syntax_highlighter.highlight(&*source_block.contents, highlighting.syntaxes());
                let html = styled_line_to_highlighted_html(&regions[..], IncludeBackground::No);

                out.push_str(&format!("<pre class=code>{}</pre>", html));
            }
        }
        Element::BabelCall(_babel_call) => {}
        Element::Section => {}
        Element::Clock(_clock) => {}
        Element::Cookie(_cookie) => {}
        Element::RadioTarget => {}
        Element::Drawer(_drawer) => {}
        Element::Document { pre_blank: _ } => {}
        Element::DynBlock(_dyn_block) => {}
        Element::FnDef(_fn_def) => {}
        Element::FnRef(fn_ref) => {
            // Anonymous definition
            if fn_ref.label == "" {
                data.footnotes.push(
                    fn_ref
                        .definition
                        .as_ref()
                        .map(|s| s.to_string())
                        .unwrap_or_default(),
                );

                out.push_str(&format!(
                    "<sup id=fns{0}><a href=#fn{0}>{0}</a></sup>",
                    data.footnotes.len()
                ));
            }
        }
        Element::Headline { level: _ } => {}
        Element::InlineCall(_inline_call) => {}
        Element::InlineSrc(_inline_src) => {}
        Element::Keyword(_keyword) => {}
        Element::Link(link) => out.push_str(&link_to_html(base_url, link)),
        Element::List(list) => {
            if list.ordered {
                out.push_str("<ol>");
            } else {
                out.push_str("<ul>");
            }
        }
        Element::ListItem(_list_item) => out.push_str("<li>"),
        Element::Macros(_macros) => {}
        Element::Snippet(_snippet) => {}
        Element::Text { value } => out.push_str(&tera::escape_html(value)),
        Element::Paragraph { post_blank: _ } => out.push_str("<p>"),
        Element::Rule(_rule) => out.push_str("<hr>"),
        Element::Timestamp(_timestamp) => {}
        Element::Target(_target) => {}
        Element::Bold => out.push_str("<i>"),
        Element::Strike => out.push_str("<s>"),
        Element::Italic => out.push_str("<i>"),
        Element::Underline => out.push_str("<u>"),
        Element::Verbatim { value: _ } => {}
        Element::Code { value: _ } => {}
        Element::Comment(_comment) => {}
        Element::FixedWidth(_fixed_width) => {}
        Element::Title(title) => {
            let level = title.level.min(6).max(1) as u8;

            let text = tera::escape_html(&title.raw);

            let headline_link = data.toc.add_headline(level, &text);

            out.push_str(&format!(
                "<h{level} id=\"{link}\"><a href=\"#{link}\">",
                level = level,
                link = &headline_link
            ));
        }
        Element::Table(_table) => {}
        Element::TableRow(_table_row) => {}
        Element::TableCell(_table_cell) => {}
    }

    Ok(())
}

fn emit_element_end(out: &mut String, element: &Element) {
    match element {
        Element::SpecialBlock(_special_block) => {}
        Element::QuoteBlock(_quote_block) => {}
        Element::CenterBlock(_center_block) => {}
        Element::VerseBlock(_verse_block) => {}
        Element::CommentBlock(_comment_block) => {}
        Element::ExampleBlock(_example_block) => {}
        Element::ExportBlock(_export_blokc) => {}
        Element::SourceBlock(_source_block) => {}
        Element::BabelCall(_babel_call) => {}
        Element::Section => {}
        Element::Clock(_clock) => {}
        Element::Cookie(_cookie) => {}
        Element::RadioTarget => {}
        Element::Drawer(_drawer) => {}
        Element::Document { pre_blank: _ } => {}
        Element::DynBlock(_dyn_block) => {}
        Element::FnDef(_fn_def) => {}
        Element::FnRef(_fn_ref) => {}
        Element::Headline { level: _ } => {}
        Element::InlineCall(_inline_call) => {}
        Element::InlineSrc(_inline_src) => {}
        Element::Keyword(_keyword) => {}
        Element::Link(_link) => out.push_str("</a>"),
        Element::List(list) => {
            if list.ordered {
                out.push_str("</ol>");
            } else {
                out.push_str("</ul>");
            }
        }
        Element::ListItem(_list_item) => out.push_str("</li>"),
        Element::Macros(_macros) => {}
        Element::Snippet(_snippet) => {}
        Element::Paragraph { post_blank: _ } => out.push_str("</p>"),
        Element::Rule(_rule) => {}
        Element::Timestamp(_timestamp) => {}
        Element::Target(_target) => {}
        Element::Bold => out.push_str("</i>"),
        Element::Strike => out.push_str("</s>"),
        Element::Italic => out.push_str("</i>"),
        Element::Underline => out.push_str("</u>"),
        Element::Verbatim { value: _ } => {}
        Element::Code { value: _ } => {}
        Element::FixedWidth(_fixed_width) => {}
        Element::Title(title) => {
            let level = title.level.min(6).max(1) as u8;

            out.push_str(&format!("</a></h{}>", level));
        }
        Element::Table(_table) => {}
        Element::TableRow(_table_row) => {}
        Element::TableCell(_table_cell) => {}
        _ => {}
    }
}

pub fn emit_document(
    document: &Org,
    base_url: &str,
    highlighting: &Highlighting,
) -> Result<(Toc, String)> {
    let mut out = String::with_capacity(1024);

    let mut data = EmitData::default();

    for event in document.iter() {
        match event {
            Event::Start(element) => {
                emit_element_start(&mut out, base_url, &mut data, element, highlighting)?
            }
            Event::End(element) => emit_element_end(&mut out, element),
        }
    }

    if !data.footnotes.is_empty() {
        out.push_str("<hr><section id=footnotes><h2>Footnotes</h2><ol>");

        for (i, footnote) in data.footnotes.iter().enumerate() {
            let fn_id = i + 1;

            out.push_str(&format!("<li id=fn{}><p>", fn_id));

            let org = Org::parse(footnote);

            for event in org.iter() {
                match event {
                    Event::Start(element) => match element {
                        Element::Link(link) => out.push_str(&link_to_html(base_url, link)),
                        Element::Text { value } => out.push_str(&tera::escape_html(value)),
                        Element::Bold => out.push_str("<i>"),
                        Element::Strike => out.push_str("<s>"),
                        Element::Italic => out.push_str("<i>"),
                        Element::Underline => out.push_str("<u>"),
                        _ => {}
                    },
                    Event::End(element) => match element {
                        Element::Link(_link) => out.push_str("</a>"),
                        Element::Bold => out.push_str("</i>"),
                        Element::Strike => out.push_str("</s>"),
                        Element::Italic => out.push_str("</i>"),
                        Element::Underline => out.push_str("</u>"),
                        _ => {}
                    },
                }
            }

            out.push_str(&format!(" <a href=#fns{}>↵</a></p></li>", fn_id));
        }

        out.push_str("</ol></section>");
    }

    Ok((data.toc, out))
}

#[derive(Error, Debug)]
pub enum OrgError {
    #[error("unknown source block language \"{0}\"")]
    UnknownSourceBlockLanguage(String),
}
