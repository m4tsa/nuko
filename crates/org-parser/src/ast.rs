use serde_derive::Serialize;
use smol_str::SmolStr;

#[derive(Debug, Serialize, Default, PartialEq)]
pub struct OrgSection {
    pub headline: Option<OrgHeadline>,
    pub children: Vec<OrgSectionContent>,
}

#[derive(Debug, Serialize, Default, PartialEq)]
pub struct OrgHeadline {
    pub level: u8,
    pub keyword: Option<SmolStr>,
    pub priority: Option<SmolStr>,
    pub content: Vec<OrgSectionContent>,
    pub tags: Option<Vec<SmolStr>>,
}

#[derive(Debug, Serialize, PartialEq)]
pub struct OrgKeyword {
    pub key: SmolStr,
    pub value: SmolStr,
}

#[derive(Debug, Serialize, PartialEq)]
pub enum OrgContent {
    Comment(String),
    Section(OrgSection),
    Keyword(OrgKeyword),
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub enum OrgSectionContent {
    Text(String),
    Bold(Vec<OrgSectionContent>),
    Italic(Vec<OrgSectionContent>),
    Underlined(Vec<OrgSectionContent>),
    Verbatim(Vec<OrgSectionContent>),
    Code(Vec<OrgSectionContent>),
    Strikethrough(Vec<OrgSectionContent>),
    Footnote {
        name: Option<String>,
        content: Vec<OrgSectionContent>,
    },
    List(OrgListEntry),
    Link {
        link: String,
        label: Vec<OrgSectionContent>,
    },
    Newline,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct OrgListEntry {
    pub ty: OrgListType,
    pub values: Vec<OrgListValue>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub enum OrgListType {
    Bullet,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub enum OrgListValue {
    Content(Vec<OrgSectionContent>),
    SubList(Box<OrgListEntry>),
}

#[derive(Default, Debug, Serialize, PartialEq)]
pub struct OrgDocument {
    pub content: Vec<OrgContent>,
}

impl OrgDocument {
    pub fn get_keyword(&self, key: &str) -> Option<&SmolStr> {
        for content in &self.content {
            if let OrgContent::Keyword(keyword) = content {
                if keyword.key == key {
                    return Some(&keyword.value);
                }
            }
        }

        None
    }
}
