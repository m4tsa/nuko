use smol_str::SmolStr;

#[derive(Debug, Default, PartialEq)]
pub struct OrgSection {
    pub headline: Option<OrgHeadline>,
    pub children: Vec<OrgSectionContent>,
}

#[derive(Debug, Default, PartialEq)]
pub struct OrgHeadline {
    pub level: u8,
    pub keyword: Option<SmolStr>,
    pub priority: Option<SmolStr>,
    pub content: Vec<OrgSectionContent>,
    pub tags: Option<Vec<SmolStr>>,
}

#[derive(Debug, PartialEq)]
pub struct OrgKeyword {
    pub key: SmolStr,
    pub value: SmolStr,
}

#[derive(Debug, PartialEq)]
pub enum OrgContent {
    Comment(String),
    Section(OrgSection),
    Keyword(OrgKeyword),
}

#[derive(Debug, PartialEq)]
pub enum OrgSectionContent {
    Text(String),
    Bold(Vec<OrgSectionContent>),
    Italic(Vec<OrgSectionContent>),
    Underlined(Vec<OrgSectionContent>),
    Verbatim(Vec<OrgSectionContent>),
    Code(Vec<OrgSectionContent>),
    Strikethrough(Vec<OrgSectionContent>),
    Newline,
}

#[derive(Default, Debug, PartialEq)]
pub struct OrgDocument {
    pub content: Vec<OrgContent>,
}
