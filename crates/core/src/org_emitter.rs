use anyhow::Result;
use nuko_org_parser::ast::{OrgContent, OrgDocument, OrgSection, OrgSectionContent};

fn emit_section_content(out: &mut String, content: &[OrgSectionContent], paragraph: bool) {
    if paragraph {
        out.push_str("<p>");
    }

    for content in content {
        match content {
            OrgSectionContent::Text(s) => out.push_str(s),
            OrgSectionContent::Bold(content) => {
                out.push_str("<b>");
                emit_section_content(out, content, false);
                out.push_str("</b>");
            }
            OrgSectionContent::Italic(content) => {
                out.push_str("<i>");
                emit_section_content(out, content, false);
                out.push_str("</i>");
            }
            OrgSectionContent::Underlined(content) => {
                out.push_str("<u>");
                emit_section_content(out, content, false);
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
                emit_section_content(out, content, false);
                out.push_str("</s>");
            }
            OrgSectionContent::Link { link, label } => {
                out.push_str(&format!("<a href=\"{}\">", link));
                emit_section_content(out, label, false);
                out.push_str("</a>");
            }
            OrgSectionContent::Newline => out.push_str("</p><p>"),
        }
    }

    if paragraph {
        out.push_str("</p>");
    }
}

fn emit_section(out: &mut String, section: &OrgSection) {
    if let Some(headline) = &section.headline {
        out.push_str(&format!("<h{}><a href=\"#\">", headline.level));

        emit_section_content(out, &headline.content, false);

        out.push_str(&format!("</a></h{}>", headline.level));
    }

    if !section.children.is_empty() {
        emit_section_content(out, &section.children, true);
    }
}

pub fn emit_document(document: &OrgDocument) -> Result<String> {
    let mut out = String::with_capacity(1024);

    for content in &document.content {
        match content {
            OrgContent::Keyword(_) => {}
            OrgContent::Comment(_) => {}
            OrgContent::Section(section) => emit_section(&mut out, section),
        }
    }

    Ok(out)
}
