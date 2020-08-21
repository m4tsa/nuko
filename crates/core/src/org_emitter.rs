use crate::toc::Toc;
use anyhow::Result;
use nuko_org_parser::ast::{
    OrgContent, OrgDocument, OrgListType, OrgListValue, OrgSection, OrgSectionContent,
};

#[derive(Default)]
pub struct EmitData {
    toc: Toc,
    footnotes: Vec<Vec<OrgSectionContent>>,
}

fn emit_section_content(
    out: &mut String,
    data: &mut Option<&mut EmitData>,
    content: &[OrgSectionContent],
    paragraph: bool,
) {
    if paragraph {
        out.push_str("<p>");
    }

    for content in content {
        match content {
            OrgSectionContent::Text(s) => out.push_str(s),
            OrgSectionContent::Bold(content) => {
                out.push_str("<b>");
                emit_section_content(out, data, content, false);
                out.push_str("</b>");
            }
            OrgSectionContent::Italic(content) => {
                out.push_str("<i>");
                emit_section_content(out, data, content, false);
                out.push_str("</i>");
            }
            OrgSectionContent::Underlined(content) => {
                out.push_str("<u>");
                emit_section_content(out, data, content, false);
                out.push_str("</u>");
            }
            OrgSectionContent::Verbatim(_content) => {
                unimplemented!("org emit verbatim");
            }
            OrgSectionContent::Code(_content) => {
                unimplemented!("org emit code");
            }
            OrgSectionContent::Strikethrough(content) => {
                out.push_str("<s>");
                emit_section_content(out, data, content, false);
                out.push_str("</s>");
            }
            OrgSectionContent::Link { link, label } => {
                out.push_str(&format!("<a href=\"{}\">", link));
                emit_section_content(out, data, label, false);
                out.push_str("</a>");
            }
            OrgSectionContent::Footnote { name, content } => {
                if name.is_some() {
                    unimplemented!("custom footnote names");
                }

                let data = data
                    .as_mut()
                    .expect("foot notes can only be used in main section");

                data.footnotes.push(content.clone());

                let id = data.footnotes.len();

                out.push_str(&format!("<sup><a href=#fn{0} id=fns{0}>{0}</a></sup>", id));
            }
            OrgSectionContent::List(list) => {
                match list.ty {
                    OrgListType::Bullet => out.push_str("<ul>"),
                };

                for value in &list.values {
                    out.push_str("<li>");
                    match value {
                        OrgListValue::Content(content) => {
                            emit_section_content(out, data, &content, false)
                        }
                        OrgListValue::SubList(_) => unimplemented!("org list sublists"),
                    }
                    out.push_str("</li>");
                }

                match list.ty {
                    OrgListType::Bullet => out.push_str("</ul>"),
                };
            }
            OrgSectionContent::Newline => {
                if paragraph {
                    out.push_str("</p><p>")
                }
            }
        }
    }

    if paragraph {
        out.push_str("</p>");
    }
}

fn content_to_str(out: &mut String, content: &[OrgSectionContent]) {
    for c in content {
        match c {
            OrgSectionContent::Text(s) => out.push_str(s),
            OrgSectionContent::Bold(c) => content_to_str(out, c),
            OrgSectionContent::Italic(c) => content_to_str(out, c),
            OrgSectionContent::Underlined(c) => content_to_str(out, c),
            OrgSectionContent::Strikethrough(c) => content_to_str(out, c),
            _ => {}
        }
    }
}

fn emit_section(out: &mut String, data: &mut EmitData, section: &OrgSection) {
    if let Some(headline) = &section.headline {
        let mut title = String::new();
        content_to_str(&mut title, &headline.content);
        let headline_link = data.toc.add_headline(headline.level, &title);

        out.push_str(&format!(
            "<h{} id=\"{1}\"><a href=\"#{1}\">",
            headline.level, &headline_link
        ));

        if headline.keyword == Some("TODO".into()) {
            out.push_str("<span class=todo>TODO</span> ");
        }

        emit_section_content(out, &mut Some(data), &headline.content, false);

        out.push_str(&format!("</a></h{}>", headline.level));
    }

    if !section.children.is_empty() {
        emit_section_content(out, &mut Some(data), &section.children, true);
    }
}

pub fn emit_document(document: &OrgDocument) -> Result<(Toc, String)> {
    let mut out = String::with_capacity(1024);

    let mut data = EmitData::default();

    for content in &document.content {
        match content {
            OrgContent::Keyword(_) => {}
            OrgContent::Comment(_) => {}
            OrgContent::Section(section) => emit_section(&mut out, &mut data, section),
        }
    }

    if !data.footnotes.is_empty() {
        out.push_str("<hr><section id=footnotes><h2>Footnotes</h2><ol>");

        for (i, footnote) in data.footnotes.iter().enumerate() {
            let fn_id = i + 1;

            out.push_str(&format!("<li id=fn{}><p>", fn_id));

            emit_section_content(&mut out, &mut None, footnote, false);

            out.push_str(&format!("  <a href=#fns{}>↵</a></p></li>", fn_id));
        }

        out.push_str("</ol></section>");
    }

    Ok((data.toc, out))
}
