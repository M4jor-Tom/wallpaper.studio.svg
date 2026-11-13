//! Which mood each config choice belongs to.
//!
//! Keys are the dotted config paths the studio's `FIELDS` already uses
//! (`background.trihex`, `icon.ship`, a mood block such as `zine`). Settings
//! inside a branch inherit the branch's moods, and an item may carry several.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Mood {
    Blueprint,
    Zine,
}

impl Mood {
    pub const ALL: [Mood; 2] = [Mood::Blueprint, Mood::Zine];

    pub fn as_str(self) -> &'static str {
        match self {
            Mood::Blueprint => "blueprint",
            Mood::Zine => "zine",
        }
    }
}

pub const MOODS: &[(&str, &[Mood])] = &[
    ("background.trihex", &[Mood::Blueprint]),
    ("background.summit", &[Mood::Zine]),
    ("background.treeline", &[Mood::Zine]),
    ("background.halfpipe", &[Mood::Zine]),
    ("icon.hexatri", &[Mood::Blueprint]),
    ("icon.ship", &[Mood::Blueprint]),
    ("overlay.matrix", &[Mood::Blueprint]),
    ("zine", &[Mood::Zine]), // the mood-wide settings block
];
