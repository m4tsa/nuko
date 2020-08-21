use serde_derive::Serialize;
use std::collections::{BTreeMap, HashSet};

#[derive(Default, Debug, Serialize)]
pub struct Toc {
    sections: BTreeMap<u32, TocSection>,
    used_shortcuts: HashSet<String>,
}

#[derive(Default, Debug, Serialize)]
pub struct TocSection {
    shortcut: String,
    text: String,
    sections: BTreeMap<u32, TocSection>,
}

impl Toc {
    fn get_section(&mut self, level: u8) -> &mut TocSection {
        let len = self.sections.len();

        if len == 0 || level == 0 {
            self.sections.insert(len as u32, Default::default());
        }

        let (_, last) = self.sections.iter_mut().last().unwrap();

        if level == 0 {
            last
        } else {
            let mut levels_left = level;

            let mut last_section = last;

            while levels_left != 0 {
                let len = last_section.sections.len();

                if len == 0 || levels_left == 1 {
                    last_section.sections.insert(len as u32, Default::default());
                }

                let (_, last_t) = last_section.sections.iter_mut().last().unwrap();
                last_section = last_t;

                levels_left -= 1;
            }

            last_section
        }
    }

    pub fn add_headline(&mut self, level: u8, title: &str) -> String {
        let mut shortcut = title
            .replace(' ', "-")
            .replace(|c: char| !c.is_alphanumeric(), "")
            .to_ascii_lowercase();

        if self.used_shortcuts.contains(&shortcut) {
            for i in 1..100 {
                let new = format!("{}{}", shortcut, i);

                if self.used_shortcuts.contains(&new) {
                    continue;
                } else {
                    shortcut = new;
                    break;
                }
            }

            assert!(!self.used_shortcuts.contains(&shortcut));
        }

        let section = self.get_section(level - 1);

        section.shortcut = shortcut.clone();
        section.text = title.into();

        shortcut
    }
}
