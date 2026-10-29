//! The rows whose button opens the desktop's file chooser and hands the path chosen to one
//! setter (`Custom::Choose`), by id. `tests/catalog_picks.rs` holds each to its QML page's
//! `FileDialog`: its title, its name filters, and the setter its `onAccepted` calls; what the
//! setter then sends is `catalog_setters.rs`'s.

use super::setter::{ANY, Args, Parse, Sends, Setter, setter, task};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Open,
    Save,
    Folder,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Start {
    Home,
    Pictures,
}

#[derive(Debug, Clone, Copy)]
pub struct Pick {
    pub id: &'static str,
    /// The button's label.
    pub button: &'static str,
    pub kind: Kind,
    /// The dialog's title.
    pub title: &'static str,
    /// The name filters, each a label and its patterns.
    pub filters: &'static [(&'static str, &'static [&'static str])],
    pub start: Start,
    /// What the path chosen is handed to.
    pub setter: Setter,
    /// The button waits for the window's job (`enabled: !Omarchy.jobBusy`).
    pub idle: bool,
}

const PNG: &[(&str, &[&str])] = &[("PNG", &["*.png"])];

pub const PICKS: &[Pick] = &[
    Pick {
        id: "background",
        button: "Choose…",
        kind: Kind::Open,
        title: "Set background",
        filters: &[(
            "Images",
            &["*.jpg", "*.jpeg", "*.png", "*.gif", "*.webp", "*.bmp"],
        )],
        start: Start::Pictures,
        setter: setter(
            "setBackgroundPath",
            Args::Given,
            ANY.text().parse(Parse::Absolute),
            Sends::Setting("background"),
        ),
        idle: false,
    },
    Pick {
        id: "plymouth",
        button: "Choose…",
        kind: Kind::Open,
        title: "Set boot logo",
        filters: PNG,
        start: Start::Pictures,
        setter: setter(
            "setPlymouthFromPath",
            Args::Given,
            ANY.text().parse(Parse::SafePath),
            Sends::Job(
                task("plymouth.set", "plymouth-set", "The boot logo")
                    .sudo()
                    .with_arg("path"),
            ),
        ),
        idle: true,
    },
    Pick {
        id: "plymouth-preview",
        button: "Preview…",
        kind: Kind::Open,
        title: "Preview boot logo",
        filters: PNG,
        start: Start::Pictures,
        setter: setter(
            "previewPlymouthFromPath",
            Args::Given,
            ANY.text().parse(Parse::SafePath),
            Sends::Job(
                task("plymouth.preview", "plymouth-preview", "The boot preview").with_arg("path"),
            ),
        ),
        idle: true,
    },
];

pub fn pick(id: &str) -> &'static Pick {
    PICKS.iter().find(|p| p.id == id).unwrap_or(&PICKS[0])
}
